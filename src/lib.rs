use core::fmt;

use proc_macro::TokenStream;
use proc_macro2::Span;
use queries::{
    InputQuery, InputSetter, InputSetterWithDurability, Queries, SetterKind, TrackedQuery,
    Transparent,
};
use quote::{format_ident, quote, ToTokens};
use syn::spanned::Spanned;
use syn::visit_mut::VisitMut;
use syn::{parse_quote, Attribute, FnArg, Path};
use syn::{ItemTrait, TraitItem};

mod queries;

#[proc_macro_attribute]
pub fn query_group(args: TokenStream, input: TokenStream) -> TokenStream {
    match query_group_impl(args, input.clone()) {
        Ok(tokens) => tokens.into(),
        Err(e) => token_stream_with_error(input, e),
    }
}

#[derive(Debug)]
struct InputStructField {
    name: proc_macro2::TokenStream,
    ty: proc_macro2::TokenStream,
}

impl fmt::Display for InputStructField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

struct SalsaAttr {
    name: String,
    tts: TokenStream,
    span: Span,
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

        let span = attr.span();

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

        Ok(SalsaAttr { name, tts, span })
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum QueryKind {
    Input,
    Tracked,
    Transparent,
}

pub(crate) fn query_group_impl(
    _args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> Result<proc_macro::TokenStream, syn::Error> {
    let mut item_trait = match syn::parse::<ItemTrait>(input) {
        Ok(path) => path,
        Err(e) => return Err(e),
    };

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
                let method_name = &method.sig.ident;
                let signature = &method.sig.clone();

                let return_type = signature.output.clone();
                match return_type {
                    syn::ReturnType::Default => continue,
                    syn::ReturnType::Type(_, expr) => {
                        let syn::Type::Path(ref expr) = *expr else {
                            continue;
                        };

                        let field = InputStructField {
                            name: method_name.to_token_stream(),
                            ty: expr.path.to_token_stream(),
                        };
                        input_struct_fields.push(field);
                    }
                }

                let (_attrs, salsa_attrs) = filter_attrs(method.attrs);

                let mut typed = None;
                let mut query_kind = QueryKind::Tracked;
                let mut invoke = None;
                let mut cycle = None;

                let params: Vec<FnArg> = signature.inputs.clone().into_iter().collect();

                // we want first query, as we later replace the receiver with a `&dyn Db`
                // in tracked functions.
                match params.as_slice() {
                    [] => return Err(syn::Error::new(signature.span(), "expected `&self`")),
                    [FnArg::Receiver(_recv)] => {}
                    [FnArg::Receiver(_recv), FnArg::Typed(ty)] => typed = Some(ty),
                    _ => continue,
                };

                let syn::ReturnType::Type(_, return_type) = &signature.output else {
                    return Err(syn::Error::new(signature.span(), "expected a return type"));
                };

                for SalsaAttr { name, tts, span } in salsa_attrs {
                    match name.as_str() {
                        "invoke" => {
                            let path = match syn::parse::<Parenthesized<Path>>(tts) {
                                Ok(path) => path,
                                Err(e) => return Err(e),
                            };
                            invoke = Some(path.0.clone());
                        }

                        "input" => {
                            if let Some(pat_type) = typed {
                                return Err(syn::Error::new(
                                    pat_type.span(),
                                    "input methods cannot have a parameter",
                                ));
                            }
                            query_kind = QueryKind::Input;
                        }
                        "transparent" => {
                            query_kind = QueryKind::Transparent;
                        }
                        "cycle" => {
                            let path = match syn::parse::<Parenthesized<Path>>(tts) {
                                Ok(path) => path,
                                Err(e) => return Err(e),
                            };
                            cycle = Some(path.0.clone())
                        }
                        _ => {
                            return Err(syn::Error::new(
                                span.clone(),
                                format!("unknown attribute `{name}`"),
                            ))
                        }
                    }
                }

                match (query_kind, invoke) {
                    // input
                    (QueryKind::Input, None) => {
                        let query = InputQuery {
                            signature: method.sig.clone(),
                        };
                        let value = Queries::InputQuery(query);
                        trait_methods.push(value);

                        let setter = InputSetter {
                            signature: method.sig.clone(),
                            return_type: *return_type.clone(),
                        };

                        setter_trait_methods.push(SetterKind::Plain(setter));

                        let setter = InputSetterWithDurability {
                            signature: method.sig.clone(),
                            return_type: *return_type.clone(),
                        };
                        setter_trait_methods.push(SetterKind::WithDurability(setter));
                    }
                    // tracked function without *any* invoke.
                    (QueryKind::Tracked, None) => {
                        let method = TrackedQuery {
                            trait_name: trait_name_ident.clone(),
                            input_struct_name: input_struct_name.clone(),
                            signature: signature.clone(),
                            typed: typed.cloned(),
                            invoke: None,
                            cycle,
                        };

                        trait_methods.push(Queries::TrackedQuery(method));
                    }
                    // tracked function with an invoke
                    (QueryKind::Tracked, Some(invoke)) => {
                        let method = TrackedQuery {
                            trait_name: trait_name_ident.clone(),
                            input_struct_name: input_struct_name.clone(),
                            signature: signature.clone(),
                            typed: typed.cloned(),
                            invoke: Some(invoke),
                            cycle,
                        };

                        trait_methods.push(Queries::TrackedQuery(method))
                    }
                    (QueryKind::Transparent, None) => {
                        let method = Transparent {
                            signature: method.sig.clone(),
                            typed: typed.unwrap().clone(),
                            invoke: None,
                        };
                        trait_methods.push(Queries::Transparent(method));
                    }
                    (QueryKind::Transparent, Some(invoke)) => {
                        let method = Transparent {
                            signature: method.sig.clone(),
                            typed: typed.unwrap().clone(),
                            invoke: Some(invoke),
                        };
                        trait_methods.push(Queries::Transparent(method));
                    }
                    (QueryKind::Input, Some(path)) => {
                        return Err(syn::Error::new(
                            path.span(),
                            format!("Inputs cannot have an `#[invoke]`"),
                        ))
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
            let ret = input.ty;
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

        #ext_trait

        #input_struct

        #create_data_method

        #trait_impl

        #ext_trait_impl
    }
    .into();

    Ok(out)
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

pub(crate) fn token_stream_with_error(mut tokens: TokenStream, error: syn::Error) -> TokenStream {
    tokens.extend(TokenStream::from(error.into_compile_error()));
    tokens
}
