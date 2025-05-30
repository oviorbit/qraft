use darling::{FromDeriveInput, FromVariant, ast};
use heck::ToSnakeCase;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{DeriveInput, parse_macro_input};

#[derive(Debug, FromDeriveInput)]
#[darling(supports(enum_unit))]
struct BinaryDeriveInput {
    ident: syn::Ident,
    data: ast::Data<BinaryVariant, ()>,
}

#[derive(Debug, FromVariant)]
#[darling(attributes(binary))]
struct BinaryVariant {
    ident: syn::Ident,
    #[darling(default)]
    ignore: bool,
}

pub fn operator_methods_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_info = match BinaryDeriveInput::from_derive_input(&input) {
        Ok(v) => v,
        Err(e) => return e.write_errors().into(),
    };

    let enum_name = &enum_info.ident;

    let variants = enum_info
        .data
        .take_enum()
        .expect("only enum is supported for now");

    let methods = variants.iter().filter_map(|var| {
        let var_name = &var.ident;
        let snake = var_name.to_string().to_snake_case();
        let where_fn = format_ident!("where_{}", snake);
        let or_where_fn = format_ident!("or_where_{}", snake);

        if var.ignore {
            return None;
        }

        Some(quote! {
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
        })
    });

    quote! {
        impl crate::Builder {
            #(#methods)*
        }
    }
    .into()
}
