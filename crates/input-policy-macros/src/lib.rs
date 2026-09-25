use std::collections::HashSet;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Data, DeriveInput, Fields, PathArguments, Token, Type, parse_macro_input, spanned::Spanned,
};

#[proc_macro_derive(Normalize, attributes(normalize))]
pub fn derive_normalize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    expand_normalize(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn expand_normalize(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = input.ident;
    let generics = input.generics;

    let fields = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => fields.named,

            _ => {
                return Err(syn::Error::new(
                    name.span(),
                    "Normalize requires a struct with named fields",
                ));
            }
        },

        _ => {
            return Err(syn::Error::new(
                name.span(),
                "Normalize can only be derived for structs",
            ));
        }
    };

    let mut normalization_steps = Vec::new();

    for field in fields {
        let Some(field_name) = field.ident else {
            continue;
        };

        let normalize_attrs: Vec<_> = field
            .attrs
            .iter()
            .filter(|attr| attr.path().is_ident("normalize"))
            .collect();

        if normalize_attrs.is_empty() {
            continue;
        }

        if !is_string(&field.ty) {
            return Err(syn::Error::new(
                field.ty.span(),
                "#[normalize(...)] currently supports String fields only",
            ));
        }

        let mut operations = HashSet::new();

        for attr in normalize_attrs {
            let mut saw_operation = false;

            attr.parse_nested_meta(|meta| {
                saw_operation = true;

                // parse_nested_meta leaves the following comma in the
                // ParseStream while this callback runs. Therefore
                // meta.input.is_empty() cannot be used to detect values.
                //
                // Explicitly reject assignment/list forms such as:
                //   trim = true
                //   trim(...)
                if meta.input.peek(Token![=]) || meta.input.peek(syn::token::Paren) {
                    return Err(meta.error("normalization operations do not accept values"));
                }

                let operation = if meta.path.is_ident("trim") {
                    "trim"
                } else if meta.path.is_ident("ascii_lowercase") {
                    "ascii_lowercase"
                } else {
                    return Err(meta.error(
                        "unsupported normalization operation; \
                         supported operations: trim, ascii_lowercase",
                    ));
                };

                if !operations.insert(operation) {
                    return Err(meta.error("duplicate normalization operation"));
                }

                match operation {
                    "trim" => {
                        normalization_steps.push(quote! {
                            self.#field_name =
                                self.#field_name.trim().to_owned();
                        });
                    }

                    "ascii_lowercase" => {
                        normalization_steps.push(quote! {
                            self.#field_name.make_ascii_lowercase();
                        });
                    }

                    _ => unreachable!(),
                }

                Ok(())
            })?;

            if !saw_operation {
                return Err(syn::Error::new(
                    attr.span(),
                    "#[normalize(...)] requires at least one operation",
                ));
            }
        }
    }

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics crate::input::Normalize
            for #name #ty_generics
            #where_clause
        {
            fn normalize(&mut self) {
                #(#normalization_steps)*
            }
        }
    })
}

fn is_string(ty: &Type) -> bool {
    let Type::Path(type_path) = ty else {
        return false;
    };

    if type_path.qself.is_some() {
        return false;
    }

    let Some(segment) = type_path.path.segments.last() else {
        return false;
    };

    segment.ident == "String" && matches!(segment.arguments, PathArguments::None)
}
