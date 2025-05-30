use darling::{ast, FromDeriveInput, FromVariant};
use heck::ToSnakeCase;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

#[derive(Debug, FromDeriveInput)]
#[darling(supports(enum_unit))]
struct BinaryDeriveInput {
    ident: syn::Ident,
    data: ast::Data<BinaryVariant, ()>,
}

#[derive(Debug, FromVariant)]
struct BinaryVariant {
    ident: syn::Ident,
}

pub fn operator_methods_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_info = match BinaryDeriveInput::from_derive_input(&input) {
        Ok(v) => v,
        Err(e) => return e.write_errors().into(),
    };

    let enum_name = &enum_info.ident;
    let methods = enum_info
        .data
        .map_enum_variants(|var| {
            let var_name = &var.ident;
            let snake = var_name.to_string().to_snake_case();
            let where_fn = format_ident!("where_{}", snake);
            let or_where_fn = format_ident!("or_where_{}", snake);

            quote! {
                impl crate::Builder {
                    pub fn #where_fn<C, V>(&mut self, column: C, value: V) -> &mut Self
                    where
                        C: crate::IntoScalarIdent,
                        V: crate::IntoScalar,
                    {
                        self.where_binary_expr(
                            crate::expr::Conjunction::And,
                            column.into_scalar_ident().0,
                            #enum_name::#var_name,
                            value.into_scalar().0,
                        )
                    }

                    pub fn #or_where_fn<C, V>(&mut self, column: C, value: V) -> &mut Self
                    where
                        C: crate::IntoScalarIdent,
                        V: crate::IntoScalar,
                    {
                        self.where_binary_expr(
                            crate::expr::Conjunction::Or,
                            column.into_scalar_ident().0,
                            #enum_name::#var_name,
                            value.into_scalar().0,
                        )
                    }
                }
            }
        })
        .take_enum()
        .unwrap_or_default();

    quote! {
        #(#methods)*
    }
    .into()
}
