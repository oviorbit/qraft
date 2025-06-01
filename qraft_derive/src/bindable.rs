// lib.rs (in your `qraft_derive` or equivalent proc‐macro crate)

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

    // 2) Ensure the input is an enum
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

    // 3) For each variant, decide whether to skip (#[bindable(ignore)]) or generate impls
    let mut all_impls = Vec::new();

    for variant in variants {
        let var_ident = &variant.ident;

        // 3a) If #[bindable(ignore)], skip entirely
        if variant.attrs.iter().any(is_bindable_ignore) {
            continue;
        }

        // 3b) Collect any other attrs (especially #[cfg(...)]).  We'll re‐emit them
        //     on each impl block.  Skip only the bindable(...) attrs here.
        let mut non_bindable_attrs = Vec::new();
        for attr in &variant.attrs {
            if !is_bindable_ignore(attr) {
                non_bindable_attrs.push(attr);
            }
        }

        // 3c) Ensure exactly one unnamed field, e.g. Variant(T) or Variant(Option<T>)
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

        // 3d) Detect Option<Inner> vs. plain Inner
        if let Some(inner_ty) = extract_inner_type_if_option(single_field_ty) {
            //  ──────────────────────────────────────────────────────────────────────────
            //  1) Generate the two `From<…> for Bind` impls
            //
            //  #[cfg(...)]? // from non_bindable_attrs
            //  impl<…> From<Inner> for Bind { … }
            //  #[cfg(...)]? // from non_bindable_attrs
            //  impl<…> From<Option<Inner>> for Bind { … }
            //
            //  2) Generate the two `IntoBind` impls
            //
            //  #[cfg(...)]? // from non_bindable_attrs
            //  impl IntoBind for Inner { … }
            //  #[cfg(...)]? // from non_bindable_attrs
            //  impl IntoBind for Option<Inner> { … }
            //  ──────────────────────────────────────────────────────────────────────────

            // (a) `impl From<Inner> for Bind`
            all_impls.push(quote! {
                #(#non_bindable_attrs)*
                impl #impl_gen From<#inner_ty> for #enum_name #ty_gen #where_clause {
                    fn from(value: #inner_ty) -> Self {
                        #enum_name::#var_ident(Some(value))
                    }
                }
            });

            // (b) `impl From<Option<Inner>> for Bind`
            all_impls.push(quote! {
                #(#non_bindable_attrs)*
                impl #impl_gen From<Option<#inner_ty>> for #enum_name #ty_gen #where_clause {
                    fn from(value: Option<#inner_ty>) -> Self {
                        #enum_name::#var_ident(value)
                    }
                }
            });

            // (c) `impl IntoBind for Inner`
            all_impls.push(quote! {
                #(#non_bindable_attrs)*
                impl IntoBind for #inner_ty {
                    fn into_bind(self) -> #enum_name {
                        #enum_name::#var_ident(Some(self))
                    }
                }
            });

            // (d) `impl IntoBind for Option<Inner>`
            all_impls.push(quote! {
                #(#non_bindable_attrs)*
                impl IntoBind for Option<#inner_ty> {
                    fn into_bind(self) -> #enum_name {
                        #enum_name::#var_ident(self)
                    }
                }
            });
        } else {
            // ────────────────────────────────────────────────────────────────────────────
            // It's a plain single field (not Option<…>), so we only generate:
            //
            // #[cfg(...)]? // from non_bindable_attrs
            // impl From<Inner> for Bind { … }
            //
            // #[cfg(...)]? // from non_bindable_attrs
            // impl IntoBind for Inner { … }
            // ────────────────────────────────────────────────────────────────────────────

            let inner_ty = single_field_ty;
            // (a) `impl From<Inner> for Bind`
            all_impls.push(quote! {
                #(#non_bindable_attrs)*
                impl #impl_gen From<#inner_ty> for #enum_name #ty_gen #where_clause {
                    fn from(value: #inner_ty) -> Self {
                        #enum_name::#var_ident(value)
                    }
                }
            });

            // (b) `impl IntoBind for Inner`
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

    // 4) Finally, emit a “fallback” impl so that `IntoBind for Bind` is itself implemented.
    //    That way `Bind::new(bind_value)` works even if you do `Bind::new(some_bind_enum)`.
    all_impls.push(quote! {
        impl IntoBind for #enum_name {
            fn into_bind(self) -> #enum_name {
                self
            }
        }
    });

    // 5) Combine everything into a single TokenStream
    let expanded = quote! {
        #(#all_impls)*
    };
    TokenStream::from(expanded)
}

/// Returns true if the attribute is exactly `#[bindable(ignore)]`.
fn is_bindable_ignore(attr: &Attribute) -> bool {
    // 1) Path must be `bindable`
    if !attr.path().is_ident("bindable") {
        return false;
    }
    // 2) Attempt to parse the inside as a single `Ident`.
    //    If it matches `ignore`, this is #[bindable(ignore)].
    matches!(attr.parse_args::<Ident>(), Ok(ident) if ident == "ignore")
}

/// If `ty` is `Option<SomeType>`, return `Some(&SomeType)`. Otherwise, None.
fn extract_inner_type_if_option(ty: &Type) -> Option<&Type> {
    // Only handle unqualified `Option<…>`.  If you want `std::option::Option`, expand here.
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
