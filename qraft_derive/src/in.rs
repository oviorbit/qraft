use darling::{ast, FromDeriveInput, FromVariant};
use heck::ToSnakeCase;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

#[derive(Debug, FromDeriveInput)]
#[darling(supports(enum_unit))]
struct InInput {
    ident: syn::Ident,
    data: ast::Data<InVariant, ()>,
}

#[derive(Debug, FromVariant)]
struct InVariant {
    ident: syn::Ident,
}

pub fn in_operator_methods_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_info = match InInput::from_derive_input(&input) {
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
                impl crate::QueryBuilder {
                    pub fn #where_fn<L, R>(&mut self, lhs: L, rhs: R) -> &mut Self
                    where
                        L: crate::IntoScalarIdent,
                        R: crate::IntoScalar,
                    {
                        self.where_in_expr(
                            crate::expr::Conjunction::And,
                            lhs.into(),
                            rhs.into(),
                            #enum_name::#var,
                        )
                    }

                    pub fn #or_where_fn<L, R>(&mut self, lhs: L, rhs: R) -> &mut Self
                    where
                        L: crate::IntoScalarIdent,
                        R: Into<SetExpr>,
                    {
                        self.where_in_expr(
                            crate::expr::Conjunction::Or,
                            lhs.into(),
                            rhs.into(),
                            #enum_name::#var,
                        )
                    }
                }
            }
        })
        .take_enum()
        .unwrap_or_default();
    quote!(#(#methods)*).into()
}
