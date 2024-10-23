use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::ItemTrait;
use syn::{parse_macro_input, parse_quote, Attribute, FnArg, Ident};

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

#[derive(Debug)]
struct QueryParams {
    receiver: syn::Receiver,
    typed: syn::PatType,
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
            syn::TraitItem::Fn(trait_item_fn) => {
                let name = &trait_item_fn.sig.ident;

                let return_type = trait_item_fn.sig.output.clone();
                match return_type {
                    syn::ReturnType::Default => continue,
                    syn::ReturnType::Type(_, expr) => {
                        let syn::Type::Path(expr) = *expr else {
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
                let params: Vec<FnArg> = trait_item_fn.clone().sig.inputs.into_iter().collect();

                // we want the receiver and the first query, as we later replace the receiver with a `&dyn Db`
                // in tracked functions.
                let [FnArg::Receiver(receiver), FnArg::Typed(typed)] = params.as_slice() else {
                    continue;
                };

                // if the attributes are empty, treat this as a normal query.
                if trait_item_fn.attrs.is_empty() {
                    let sig = trait_item_fn.sig;
                    let ret = sig.output.clone();
                    let param = typed.clone().pat;
                    let method = quote! {
                        #sig {
                            fn __shim__(
                                db: &dyn #trait_name,
                                _input: #input_struct_name,
                                #typed
                            ) #ret {
                                todo!()
                            }
                            __shim__(self, create_data(self), #param)
                        }
                    };
                    trait_methods.push(method);
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

    eprintln!("trait_impl: {}", trait_impl);

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
