use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Attribute, Data, DeriveInput, Error, Fields, GenericArgument, Ident,
    PathArguments, Type,
};

pub fn bindable_derive_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_name = &input.ident;
    let generics = &input.generics;
    let (impl_gen, ty_gen, where_clause) = generics.split_for_impl();

    let variants = match &input.data {
        Data::Enum(e) => &e.variants,
        _ => {
            return Error::new_spanned(
                &input.ident,
                "`#[derive(Bindable)]` can only be applied to enums",
            )
            .to_compile_error()
            .into()
        }
    };

    let mut all_impls = Vec::new();

    for variant in variants {
        let var_ident = &variant.ident;
        if variant.attrs.iter().any(is_bindable_ignore) {
            continue;
        }

        let mut non_bindable_attrs = Vec::new();
        for attr in &variant.attrs {
            if !is_bindable_ignore(attr) {
                non_bindable_attrs.push(attr);
            }
        }

        let single_field_ty = match &variant.fields {
            Fields::Unnamed(fields_unnamed) if fields_unnamed.unnamed.len() == 1 => {
                &fields_unnamed.unnamed.first().unwrap().ty
            }
            _ => {
                let err = Error::new_spanned(
                    &variant.ident,
                    "Each non-ignored variant must be a single unnamed field, e.g. `Foo(Option<T>)` or `Bar(T)`.",
                );
                return err.to_compile_error().into();
            }
        };

        if let Some(inner_ty) = extract_inner_type_if_option(single_field_ty) {
            all_impls.push(quote! {
                #(#non_bindable_attrs)*
                impl #impl_gen From<#inner_ty> for #enum_name #ty_gen #where_clause {
                    fn from(value: #inner_ty) -> Self {
                        #enum_name::#var_ident(Some(value))
                    }
                }
            });

            all_impls.push(quote! {
                #(#non_bindable_attrs)*
                impl #impl_gen From<Option<#inner_ty>> for #enum_name #ty_gen #where_clause {
                    fn from(value: Option<#inner_ty>) -> Self {
                        #enum_name::#var_ident(value)
                    }
                }
            });

            all_impls.push(quote! {
                #(#non_bindable_attrs)*
                impl IntoBind for #inner_ty {
                    fn into_bind(self) -> #enum_name {
                        #enum_name::#var_ident(Some(self))
                    }
                }
            });
            all_impls.push(quote! {
                #(#non_bindable_attrs)*
                impl IntoBind for Option<#inner_ty> {
                    fn into_bind(self) -> #enum_name {
                        #enum_name::#var_ident(self)
                    }
                }
            });
        } else {
            let inner_ty = single_field_ty;
            all_impls.push(quote! {
                #(#non_bindable_attrs)*
                impl #impl_gen From<#inner_ty> for #enum_name #ty_gen #where_clause {
                    fn from(value: #inner_ty) -> Self {
                        #enum_name::#var_ident(value)
                    }
                }
            });

            all_impls.push(quote! {
                #(#non_bindable_attrs)*
                impl IntoBind for #inner_ty {
                    fn into_bind(self) -> #enum_name {
                        #enum_name::#var_ident(self)
                    }
                }
            });
        }
    }

    all_impls.push(quote! {
        impl IntoBind for #enum_name {
            fn into_bind(self) -> #enum_name {
                self
            }
        }
    });

    let expanded = quote! {
        #(#all_impls)*
    };
    TokenStream::from(expanded)
}

fn is_bindable_ignore(attr: &Attribute) -> bool {
    if !attr.path().is_ident("bindable") {
        return false;
    }
    matches!(attr.parse_args::<Ident>(), Ok(ident) if ident == "ignore")
}

fn extract_inner_type_if_option(ty: &Type) -> Option<&Type> {
    if let Type::Path(type_path) = ty {
        if let Some(last_seg) = type_path.path.segments.last() {
            if last_seg.ident == "Option" {
                if let PathArguments::AngleBracketed(angle_args) = &last_seg.arguments {
                    if angle_args.args.len() == 1 {
                        if let GenericArgument::Type(inner_ty) = angle_args.args.first().unwrap() {
                            return Some(inner_ty);
                        }
                    }
                }
            }
        }
    }
    None
}
