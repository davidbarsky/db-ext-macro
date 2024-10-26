use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::visit_mut::VisitMut;
use syn::{
    parse_macro_input, parse_quote, Attribute, FnArg, Ident, PatType, Path, Receiver, ReturnType,
    Signature,
};
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

struct TrackedQuery {
    trait_name: Ident,
    input_struct_name: Ident,
    signature: Signature,
    typed: PatType,
}

impl ToTokens for TrackedQuery {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let sig = &self.signature;
        let trait_name = &self.trait_name;
        let input_struct_name = &self.input_struct_name;
        let ret = &sig.output;
        let type_ascription = &self.typed;
        let typed = &self.typed.pat;
        let invoke = &sig.ident;

        let method = quote! {
            #sig {
                fn __shim__(
                    db: &dyn #trait_name,
                    _input: #input_struct_name,
                    #type_ascription,
                ) #ret {
                    #invoke(db, #typed)
                }
                __shim__(self, create_data(self), #typed)
            }
        };

        method.to_tokens(tokens);
    }
}

struct TrackedInvokeQuery {
    trait_name: Ident,
    input_struct_name: Ident,
    signature: Signature,
    typed: PatType,
    invoke: Path,
}

impl ToTokens for TrackedInvokeQuery {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let sig = &self.signature;
        let trait_name = &self.trait_name;
        let input_struct_name = &self.input_struct_name;
        let ret = &sig.output;
        let type_ascription = &self.typed;
        let typed = &self.typed.pat;
        let invoke = self.invoke.clone();

        let method = quote! {
            #sig {
                fn __shim__(
                    db: &dyn #trait_name,
                    _input: #input_struct_name,
                    #type_ascription,
                ) #ret {
                    #invoke(db, #typed)
                }
                __shim__(self, create_data(self), #typed)
            }
        };
        method.to_tokens(tokens);
    }
}

struct InputQuery {
    signature: Signature,
}

impl ToTokens for InputQuery {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let sig = &self.signature;
        let fn_ident = &sig.ident;

        let method = quote! {
            #sig {
                let data = create_data(self);
                data.#fn_ident(self).unwrap()
            }
        };
        method.to_tokens(tokens);
    }
}

struct InputSetter {
    signature: Signature,
}

impl ToTokens for InputSetter {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let sig = &self.signature;
        let fn_ident = &sig.ident;
        let syn::ReturnType::Type(_, ty) = &sig.output else {
            return;
        };

        let mut sig = self.signature.clone();
        sig.ident = format_ident!("set_{}", fn_ident);
        let sig_ident = &sig.ident;

        let value_argument: PatType = parse_quote!(__value: #ty);
        let pat = &value_argument.pat.clone();
        sig.inputs.push(FnArg::Typed(value_argument));

        // make `&self` `&mut self` instead.
        let mut_recevier: Receiver = parse_quote!(&mut self);
        sig.inputs
            .first_mut()
            .map(|og| *og = FnArg::Receiver(mut_recevier));

        // remove the return value.
        sig.output = ReturnType::Default;

        let method = quote! {
            #sig {
                let data = create_data(self);
                data.#sig_ident(self).to(Some(#pat));
            }
        };
        method.to_tokens(tokens);
    }
}

struct InputSetterWithDurability {
    signature: Signature,
}

impl ToTokens for InputSetterWithDurability {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let sig = &self.signature;
        let fn_ident = &sig.ident;
        let syn::ReturnType::Type(_, ty) = &sig.output else {
            return;
        };

        let mut sig = self.signature.clone();
        sig.ident = format_ident!("set_{}_with_durability", fn_ident);
        let sig_ident = format_ident!("set_{}", fn_ident);

        let value_argument: PatType = parse_quote!(__value: #ty);
        let value_pat = &value_argument.pat.clone();
        sig.inputs.push(FnArg::Typed(value_argument));

        let durability_argument: PatType = parse_quote!(durability: salsa::Durability);
        let durability_pat = &durability_argument.pat.clone();
        sig.inputs.push(FnArg::Typed(durability_argument));

        // make `&self` `&mut self` instead.
        let mut_recevier: Receiver = parse_quote!(&mut self);
        sig.inputs
            .first_mut()
            .map(|og| *og = FnArg::Receiver(mut_recevier));

        // remove the return value.
        sig.output = ReturnType::Default;

        let method = quote! {
            #sig {
                let data = create_data(self);
                data.#sig_ident(self)
                    .with_durability(#durability_pat)
                    .to(Some(#value_pat));
            }
        };
        eprintln!("{}", method);
        method.to_tokens(tokens);
    }
}

enum SetterKind {
    Plain(InputSetter),
    WithDurability(InputSetterWithDurability),
}

impl ToTokens for SetterKind {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            SetterKind::Plain(input_setter) => input_setter.to_tokens(tokens),
            SetterKind::WithDurability(input_setter_with_durability) => {
                input_setter_with_durability.to_tokens(tokens)
            }
        }
    }
}

// struct TransparentMethod {}

enum Queries {
    TrackedQuery(TrackedQuery),
    TrackedInvokeQuery(TrackedInvokeQuery),
    InputQuery(InputQuery),
}

impl ToTokens for Queries {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Queries::TrackedQuery(tracked_query) => tracked_query.to_tokens(tokens),
            Queries::TrackedInvokeQuery(tracked_invoke_query) => {
                tracked_invoke_query.to_tokens(tokens)
            }
            Queries::InputQuery(input_query) => input_query.to_tokens(tokens),
        }
    }
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

    let trait_name_ident = &item_trait.ident.clone();
    let input_struct_name = format_ident!("{}Data", trait_name_ident);

    let mut input_struct_fields: Vec<InputStructField> = vec![];
    let mut trait_methods = vec![];
    let mut setter_trait_methods = vec![];

    for item in item_trait.clone().items {
        match item {
            syn::TraitItem::Fn(method) => {
                let name = &method.sig.ident;
                let signature = &method.sig;

                let return_type = signature.output.clone();
                match return_type {
                    syn::ReturnType::Default => continue,
                    syn::ReturnType::Type(_, expr) => {
                        let syn::Type::Path(ref expr) = *expr else {
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
                let params: Vec<FnArg> = signature.inputs.clone().into_iter().collect();

                // we want first query, as we later replace the receiver with a `&dyn Db`
                // in tracked functions.
                let [FnArg::Receiver(_), FnArg::Typed(typed)] = params.as_slice() else {
                    continue;
                };

                let (_attrs, salsa_attrs) = filter_attrs(method.attrs);

                if salsa_attrs.is_empty() {
                    let method = TrackedQuery {
                        trait_name: trait_name_ident.clone(),
                        input_struct_name: input_struct_name.clone(),
                        signature: method.sig.clone(),
                        typed: typed.clone(),
                    };

                    trait_methods.push(Queries::TrackedQuery(method));
                } else {
                    for SalsaAttr { name, tts, .. } in salsa_attrs {
                        match name.as_str() {
                            "invoke" => {
                                let invoke = parse_macro_input!(tts as Parenthesized<syn::Path>).0;

                                let method = TrackedInvokeQuery {
                                    trait_name: trait_name_ident.clone(),
                                    input_struct_name: input_struct_name.clone(),
                                    signature: method.sig.clone(),
                                    typed: typed.clone(),
                                    invoke,
                                };

                                trait_methods.push(Queries::TrackedInvokeQuery(method));
                            }
                            "input" => {
                                let query = InputQuery {
                                    signature: method.sig.clone(),
                                };
                                trait_methods.push(Queries::InputQuery(query));

                                let setter = InputSetter {
                                    signature: method.sig.clone(),
                                };
                                setter_trait_methods.push(SetterKind::Plain(setter));

                                let setter = InputSetterWithDurability {
                                    signature: method.sig.clone(),
                                };
                                setter_trait_methods.push(SetterKind::WithDurability(setter));
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
        fn create_data(db: &dyn #trait_name_ident) -> #input_struct_name {
            #input_struct_name::new(db, #(#field_params),*)
        }
    };

    let trait_impl = quote! {
        #[salsa::db]
        impl<DB> #trait_name_ident for DB
        where
            DB: salsa::Database,
        {
            #(#trait_methods)*
        }
    };
    RemoveAttrsFromTraitMethods.visit_item_trait_mut(&mut item_trait);

    let ext_trait_ident = format_ident!("{}SetterExt", trait_name_ident);
    let ext_trait: ItemTrait = parse_quote! {
        trait #ext_trait_ident: #trait_name_ident
        where
            Self: Sized,
        {
            #(#setter_trait_methods)*
        }
    };

    let ext_trait_impl = quote! {
        use salsa::Setter;

        impl<DB> #ext_trait_ident for DB
        where
            DB: #trait_name_ident,
        {

        }
    };

    let out = quote! {
        #item_trait

        // const hidden: () = {
            #input_struct

            #create_data_method

            #trait_impl

            #ext_trait

            #ext_trait_impl
        // };
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
