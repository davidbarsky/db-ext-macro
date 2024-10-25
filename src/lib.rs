use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::visit_mut::VisitMut;
use syn::{parse_macro_input, parse_quote, Attribute, FnArg, Ident, Path};
use syn::{ItemTrait, TraitItem};

#[proc_macro_attribute]
pub fn db_ext(args: TokenStream, input: TokenStream) -> TokenStream {
    db_ext_impl(args, input)
}

#[proc_macro_attribute]
pub fn input(_args: TokenStream, input: TokenStream) -> TokenStream {
    input
}

#[derive(Debug)]
struct InputStructField {
    name: Ident,
    return_type: Ident,
}

struct SalsaAttr {
    name: String,
    tts: TokenStream,
}

impl std::fmt::Debug for SalsaAttr {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "{:?}", self.name)
    }
}

impl TryFrom<syn::Attribute> for SalsaAttr {
    type Error = syn::Attribute;

    fn try_from(attr: syn::Attribute) -> Result<SalsaAttr, syn::Attribute> {
        if is_not_salsa_attr_path(attr.path()) {
            return Err(attr);
        }

        let name = attr.path().segments[1].ident.to_string();
        let tts = match attr.meta {
            syn::Meta::Path(path) => path.into_token_stream(),
            syn::Meta::List(ref list) => {
                let tts = list
                    .into_token_stream()
                    .into_iter()
                    .skip(attr.path().to_token_stream().into_iter().count());
                proc_macro2::TokenStream::from_iter(tts)
            }
            syn::Meta::NameValue(nv) => nv.into_token_stream(),
        }
        .into();

        Ok(SalsaAttr { name, tts })
    }
}

fn is_not_salsa_attr_path(path: &syn::Path) -> bool {
    path.segments
        .first()
        .map(|s| s.ident != "db_ext_macro")
        .unwrap_or(true)
        || path.segments.len() != 2
}

fn filter_attrs(attrs: Vec<Attribute>) -> (Vec<Attribute>, Vec<SalsaAttr>) {
    let mut other = vec![];
    let mut ra_salsa = vec![];
    // Leave non-ra_salsa attributes untouched. These are
    // attributes that don't start with `ra_salsa::` or don't have
    // exactly two segments in their path.
    // Keep the ra_salsa attributes around.
    for attr in attrs {
        match SalsaAttr::try_from(attr) {
            Ok(it) => ra_salsa.push(it),
            Err(it) => other.push(it),
        }
    }
    (other, ra_salsa)
}

pub(crate) fn db_ext_impl(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut item_trait: ItemTrait = parse_macro_input!(input as ItemTrait);
    let db_attr: Attribute = parse_quote! {
        #[salsa::db]
    };
    item_trait.attrs.push(db_attr);

    let trait_name = &item_trait.ident.clone();
    let input_struct_name = format_ident!("{}Data", trait_name);

    let mut input_struct_fields: Vec<InputStructField> = vec![];
    let mut trait_methods = vec![];

    for item in item_trait.clone().items {
        match item {
            syn::TraitItem::Fn(method) => {
                let name = &method.sig.ident;

                let return_type = &method.sig.output.clone();
                match return_type {
                    syn::ReturnType::Default => continue,
                    syn::ReturnType::Type(_, expr) => {
                        let syn::Type::Path(ref expr) = **expr else {
                            continue;
                        };

                        let Some(ident) = expr.path.get_ident() else {
                            continue;
                        };

                        let field = InputStructField {
                            name: name.clone(),
                            return_type: ident.clone(),
                        };
                        input_struct_fields.push(field);
                    }
                }
                let params: Vec<FnArg> = method.clone().sig.inputs.into_iter().collect();

                // we want first query, as we later replace the receiver with a `&dyn Db`
                // in tracked functions.
                let [FnArg::Receiver(_), FnArg::Typed(typed)] = params.as_slice() else {
                    continue;
                };

                let (_attrs, salsa_attrs) = filter_attrs(method.attrs);
                let sig = &method.sig;
                let ret = &sig.output;
                let param = &typed.pat;

                if salsa_attrs.is_empty() {
                    let invoke = &sig.ident;
                    let method = quote! {
                        #sig {
                            fn __shim__(
                                db: &dyn #trait_name,
                                _input: #input_struct_name,
                                #typed
                            ) #ret {
                                #invoke(db, #param)
                            }
                            __shim__(self, create_data(self), #param)
                        }
                    };
                    trait_methods.push(method);
                } else {
                    for SalsaAttr { name, tts, .. } in salsa_attrs {
                        match name.as_str() {
                            "invoke" => {
                                let invoke = parse_macro_input!(tts as Parenthesized<syn::Path>).0;
                                let method = quote! {
                                    #sig {
                                        fn __shim__(
                                            db: &dyn #trait_name,
                                            _input: #input_struct_name,
                                            #typed
                                        ) #ret {
                                            #invoke(db, #param)
                                        }
                                        __shim__(self, create_data(self), #param)
                                    }
                                };
                                trait_methods.push(method);
                            }
                            "input" => {
                                // let ident = &sig.ident;
                                // eprintln!("{:?}", receiver);
                                // let input_query = quote! {
                                //     #sig {
                                //         let data = create_data(self);
                                //         data.#ident(#receiver).unwrap()
                                //     }
                                // };
                                // eprintln!("{}", input_query);
                                // trait_methods.push(input_query);
                            }
                            _ => continue,
                        }
                    }
                }
            }

            _ => (),
        }
    }

    let fields = input_struct_fields
        .into_iter()
        .map(|input| {
            let name = input.name;
            let ret = input.return_type;
            quote! { #name: Option<#ret> }
        })
        .collect::<Vec<proc_macro2::TokenStream>>();

    let input_struct = quote! {
        #[salsa::input]
        struct #input_struct_name {
            #(#fields),*
        }
    };
    let field_params = std::iter::repeat_n(quote! { None }, fields.len())
        .collect::<Vec<proc_macro2::TokenStream>>();

    let create_data_method = quote! {
        #[salsa::tracked]
        fn create_data(db: &dyn #trait_name) -> #input_struct_name {
            #input_struct_name::new(db, #(#field_params),*)
        }
    };

    let trait_impl = quote! {
        #[salsa::db]
        impl<DB> #trait_name for DB
        where
            DB: salsa::Database,
        {
            #(#trait_methods)*
        }
    };

    RemoveAttrsFromTraitMethods.visit_item_trait_mut(&mut item_trait);

    let out = quote! {
        #item_trait

        const _: () = {
            #input_struct

            #create_data_method

            #trait_impl
        };
    }
    .into();

    out
}

/// Parenthesis helper
pub(crate) struct Parenthesized<T>(pub(crate) T);

impl<T> syn::parse::Parse for Parenthesized<T>
where
    T: syn::parse::Parse,
{
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let content;
        syn::parenthesized!(content in input);
        content.parse::<T>().map(Parenthesized)
    }
}

struct RemoveAttrsFromTraitMethods;

impl VisitMut for RemoveAttrsFromTraitMethods {
    fn visit_item_trait_mut(&mut self, i: &mut syn::ItemTrait) {
        for item in &mut i.items {
            match item {
                TraitItem::Fn(trait_item_fn) => {
                    trait_item_fn.attrs = vec![];
                }
                _ => (),
            }
        }
    }
}
