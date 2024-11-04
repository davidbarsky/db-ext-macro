use quote::{format_ident, quote, ToTokens};
use syn::{parse_quote, FnArg, Ident, PatType, Path, Receiver, ReturnType, Signature};

pub(crate) struct TrackedQuery {
    pub(crate) trait_name: Ident,
    pub(crate) input_struct_name: Ident,
    pub(crate) signature: Signature,
    pub(crate) typed: PatType,
    pub(crate) invoke: Option<Path>,
}

impl ToTokens for TrackedQuery {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let sig = &self.signature;
        let trait_name = &self.trait_name;
        let input_struct_name = &self.input_struct_name;
        let ret = &sig.output;
        let type_ascription = &self.typed;
        let typed = &self.typed.pat;

        let invoke = match &self.invoke {
            Some(path) => path.to_token_stream(),
            None => sig.ident.to_token_stream(),
        };

        let fn_ident = &sig.ident;
        let shim: Ident = format_ident!("{}_shim", fn_ident);

        let method = quote! {
            #sig {
                #[salsa::tracked]
                fn #shim(
                    db: &dyn #trait_name,
                    _input: #input_struct_name,
                    #type_ascription,
                ) #ret {
                    #invoke(db, #typed)
                }
                #shim(self, create_data(self), #typed)
            }
        };
        method.to_tokens(tokens);
    }
}

pub(crate) struct InputQuery {
    pub(crate) signature: Signature,
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

pub(crate) struct InputSetter {
    pub(crate) signature: Signature,
    pub(crate) return_type: syn::Type,
}

impl ToTokens for InputSetter {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let sig = &mut self.signature.clone();

        let ty = &self.return_type;
        let fn_ident = &sig.ident;

        let setter_ident = format_ident!("set_{}", fn_ident);
        sig.ident = setter_ident.clone();

        let value_argument: PatType = parse_quote!(__value: #ty);
        sig.inputs.push(FnArg::Typed(value_argument.clone()));

        // make `&self` `&mut self` instead.
        let mut_recevier: Receiver = parse_quote!(&mut self);
        sig.inputs
            .first_mut()
            .map(|og| *og = FnArg::Receiver(mut_recevier));

        // remove the return value.
        sig.output = ReturnType::Default;

        let value = &value_argument.pat;
        let method = quote! {
            #sig {
                let data = create_data(self);
                data.#setter_ident(self).to(Some(#value));
            }
        };
        method.to_tokens(tokens);
    }
}

pub(crate) struct InputSetterWithDurability {
    pub(crate) signature: Signature,
    pub(crate) return_type: syn::Type,
}

impl ToTokens for InputSetterWithDurability {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let sig = &mut self.signature.clone();

        let ty = &self.return_type;
        let fn_ident = &sig.ident;
        let setter_ident = format_ident!("set_{}", fn_ident);

        sig.ident = format_ident!("set_{}_with_durability", fn_ident);

        let value_argument: PatType = parse_quote!(__value: #ty);
        sig.inputs.push(FnArg::Typed(value_argument.clone()));

        let durability_argument: PatType = parse_quote!(durability: salsa::Durability);
        sig.inputs.push(FnArg::Typed(durability_argument.clone()));

        // make `&self` `&mut self` instead.
        let mut_recevier: Receiver = parse_quote!(&mut self);
        sig.inputs
            .first_mut()
            .map(|og| *og = FnArg::Receiver(mut_recevier));

        // remove the return value.
        sig.output = ReturnType::Default;

        let value = &value_argument.pat;
        let durability = &durability_argument.pat;
        let method = quote! {
            #sig {
                let data = create_data(self);
                data.#setter_ident(self)
                    .with_durability(#durability)
                    .to(Some(#value));
            }
        };
        method.to_tokens(tokens);
    }
}

pub(crate) enum SetterKind {
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

pub(crate) struct Transparent {
    pub(crate) signature: Signature,
    pub(crate) typed: PatType,
    pub(crate) invoke: Option<Path>,
}

impl ToTokens for Transparent {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let sig = &self.signature;
        let typed = &self.typed.pat;

        let invoke = match &self.invoke {
            Some(path) => path.to_token_stream(),
            None => sig.ident.to_token_stream(),
        };

        let method = quote! {
            #sig {
                #invoke(self, #typed)
            }
        };

        method.to_tokens(tokens);
    }
}

pub(crate) enum Queries {
    TrackedQuery(TrackedQuery),
    InputQuery(InputQuery),
    Transparent(Transparent),
}

impl ToTokens for Queries {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Queries::TrackedQuery(tracked_query) => tracked_query.to_tokens(tokens),
            Queries::InputQuery(input_query) => input_query.to_tokens(tokens),
            Queries::Transparent(transparent) => transparent.to_tokens(tokens),
        }
    }
}
