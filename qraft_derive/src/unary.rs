use darling::{ast, FromDeriveInput, FromVariant};
use heck::ToSnakeCase;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

#[derive(Debug, FromDeriveInput)]
#[darling(supports(enum_unit))]
struct UnaryInput {
    ident: syn::Ident,
    data: ast::Data<UnaryVariant, ()>,
}

#[derive(Debug, FromVariant)]
struct UnaryVariant {
    ident: syn::Ident,
}

pub fn unary_operator_methods_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let op = match UnaryInput::from_derive_input(&input) {
        Ok(v) => v,
        Err(e) => return e.write_errors().into(),
    };

    let enum_name = &op.ident;
    let methods = op
        .data
        .map_enum_variants(|v| {
            let var = &v.ident;
            let snake = var.to_string().to_snake_case();
            let where_fn = format_ident!("where_{}", snake);
            let or_where_fn = format_ident!("or_where_{}", snake);

            quote! {
                impl crate::Builder {
                    pub fn #where_fn<C>(&mut self, column: C) -> &mut Self
                    where
                        C: crate::IntoScalarIdent,
                    {
                        self.where_unary_expr(
                            crate::expr::Conjunction::And,
                            column.into_scalar_ident(),
                            #enum_name::#var,
                        )
                    }

                    pub fn #or_where_fn<C>(&mut self, column: C) -> &mut Self
                    where
                        C: crate::IntoScalarIdent
                    {
                        self.where_unary_expr(
                            crate::expr::Conjunction::And,
                            column.into_scalar_ident(),
                            #enum_name::#var,
                        )
                    }
                }
            }
        })
        .take_enum()
        .unwrap_or_default();

    quote! { #(#methods)* }.into()
}
