use darling::{ast, FromDeriveInput, FromVariant};
use heck::ToSnakeCase;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

#[derive(Debug, FromDeriveInput)]
#[darling(supports(enum_unit))]
struct ExistsInput {
    ident: syn::Ident,
    data: ast::Data<ExistsVariant, ()>,
}

#[derive(Debug, FromVariant)]
struct ExistsVariant {
    ident: syn::Ident,
}

pub fn exists_operator_methods_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_info = match ExistsInput::from_derive_input(&input) {
        Ok(v) => v,
        Err(e) => return e.write_errors().into(),
    };
    let enum_name = &enum_info.ident;
    let methods = enum_info
        .data
        .map_enum_variants(|v| {
            let var = &v.ident;
            let snake = var.to_string().to_snake_case();
            let where_fn = format_ident!("where_{}", snake);
            let or_where_fn = format_ident!("or_where_{}", snake);
            quote! {
                pub fn #where_fn<Q>(&mut self, rhs: Q) -> &mut Self
                where
                    Q: crate::IntoBuilder
                {
                    self.where_exists_expr(
                        crate::expr::Conjunction::And,
                        #enum_name::#var,
                        rhs.into_builder(),
                    )
                }

                pub fn #or_where_fn<Q>(&mut self, rhs: Q) -> &mut Self
                where
                    Q: crate::IntoBuilder
                {
                    self.where_exists_expr(
                        crate::expr::Conjunction::Or,
                        #enum_name::#var,
                        rhs.into_builder(),
                    )
                }
            }
        })
        .take_enum()
        .unwrap_or_default();

    quote! {
        impl crate::Builder {
            #(#methods)*
        }
    }.into()
}
