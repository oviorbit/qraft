use std::collections::VecDeque;

use heck::ToPascalCase;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, Ident};

mod bindable;

#[proc_macro_derive(Bindable, attributes(bindable))]
pub fn bindable_derive(input: TokenStream) -> TokenStream {
    bindable::bindable_derive_impl(input)
}

#[proc_macro_attribute]
pub fn or_variant(attr: TokenStream, item: TokenStream) -> TokenStream {
    let raw = attr.to_string();
    let trimmed = raw.trim().trim_start_matches('(').trim_end_matches(')');
    let args: VecDeque<String> = trimmed
        .split(',')
        .map(|s| s.trim().trim_matches('"').to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let input_fn = parse_macro_input!(item as ItemFn);

    let fn_vis = &input_fn.vis;
    let fn_unsafety = &input_fn.sig.unsafety;
    let fn_asyncness = &input_fn.sig.asyncness;

    let fn_name = &input_fn.sig.ident;
    let fn_generics = &input_fn.sig.generics;
    let fn_inputs = &input_fn.sig.inputs;
    let fn_output = &input_fn.sig.output;
    let fn_where = &input_fn.sig.generics.where_clause;
    let fn_block = &input_fn.block;

    let or_name_string = format!("or_{}", fn_name);
    let or_name_string = or_name_string.replace("r#", "");
    let or_fn_name = Ident::new(&or_name_string, fn_name.span());

    let original_block_tokens: proc_macro2::TokenStream = quote! { #fn_block };
    let original_block_string = original_block_tokens.to_string();

    let mut args_iter = args.iter();
    let first_arg = args_iter.next();

    let or_block_string = if !args.is_empty() && first_arg.is_some_and(|v| v == "not") {
        original_block_string.replacen("Conjunction :: AndNot", "Conjunction :: OrNot", 1)
    } else {
        original_block_string.replacen("Conjunction :: And", "Conjunction :: Or", 1)
    };

    let or_block_tokens: proc_macro2::TokenStream = or_block_string
        .parse()
        .expect("Failed to re‐parse function body with Or replacement");

    let expanded = quote! {
        #fn_vis #fn_unsafety #fn_asyncness fn #fn_name #fn_generics ( #fn_inputs ) #fn_output
        #fn_where
        {
            #fn_block
        }

        #fn_vis #fn_unsafety #fn_asyncness fn #or_fn_name #fn_generics ( #fn_inputs ) #fn_output
        #fn_where
        {
            #or_block_tokens
        }
    };

    expanded.into()
}

#[proc_macro_attribute]
pub fn condition_variant(attr: TokenStream, item: TokenStream) -> TokenStream {
    let raw = attr.to_string();
    let trimmed = raw.trim().trim_start_matches('(').trim_end_matches(')');
    let args: VecDeque<String> = trimmed
        .split(',')
        .map(|s| s.trim().trim_matches('"').to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let mut args_iter = args.iter();

    let input_fn = parse_macro_input!(item as ItemFn);

    let fn_vis = &input_fn.vis;
    let fn_unsafety = &input_fn.sig.unsafety;
    let fn_asyncness = &input_fn.sig.asyncness;

    let fn_name = &input_fn.sig.ident;
    let fn_generics = &input_fn.sig.generics;
    let fn_inputs = &input_fn.sig.inputs;
    let fn_output = &input_fn.sig.output;
    let fn_where = &input_fn.sig.generics.where_clause;
    let fn_block = &input_fn.block;

    let name_string = format!("{}", fn_name).replace("where", "having");
    let having_fn_name = Ident::new(&name_string, fn_name.span());

    let original_block_tokens: proc_macro2::TokenStream = quote! { #fn_block };
    let original_block_string = original_block_tokens.to_string();
    let having_block_string = original_block_string.replace("where", "having").replace("Where", "Having");
    let having_block_tokens: proc_macro2::TokenStream = having_block_string
        .parse()
        .expect("Failed to re‐parse function body with Or replacement");

    let first_arg = args_iter.next();
    let tag = if first_arg.is_some_and(|v| v == "not") {
        quote! {
            #[or_variant(not)]
        }
    } else if first_arg.is_some_and(|v| v == "none") {
        quote! {}
    } else {
        quote! {
            #[or_variant]
        }
    };

    let expanded = quote! {
        #tag
        #fn_vis #fn_unsafety #fn_asyncness fn #fn_name #fn_generics ( #fn_inputs ) #fn_output
        #fn_where
        {
            #fn_block
        }

        #tag
        #fn_vis #fn_unsafety #fn_asyncness fn #having_fn_name #fn_generics ( #fn_inputs ) #fn_output
        #fn_where
        {
            #having_block_tokens
        }
    };

    expanded.into()
}

#[proc_macro_attribute]
pub fn variant(attr: TokenStream, item: TokenStream) -> TokenStream {
    let raw = attr.to_string();
    let trimmed = raw.trim().trim_start_matches('(').trim_end_matches(')');
    let mut snakes: VecDeque<String> = trimmed
        .split(',')
        .map(|s| s.trim().trim_matches('"').to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let enum_name = snakes.pop_front().expect("expect the enum name");
    let mut is_join = false;
    let mut is_none = false;
    let enum_name = if enum_name == "join" {
        is_join = true;
        snakes.pop_front().expect("expect the enum name")
    } else if enum_name == "none" {
        is_none = true;
        snakes.pop_front().expect("expect the enum name")
    } else {
        enum_name
    };
    let variant_name = snakes.pop_front().expect("expect the enum variant name");
    if snakes.len() < 1 {
        panic!("expected at least one operator inside #[binary(...)]");
    }

    let input_fn = parse_macro_input!(item as ItemFn);

    let fn_vis = &input_fn.vis;
    let fn_unsafety = &input_fn.sig.unsafety;
    let fn_asyncness = &input_fn.sig.asyncness;

    let fn_name = &input_fn.sig.ident;
    let fn_generics = &input_fn.sig.generics;
    let fn_inputs = &input_fn.sig.inputs;
    let fn_output = &input_fn.sig.output;
    let fn_where = &input_fn.sig.generics.where_clause;
    let fn_block = &input_fn.block;

    let fns: Vec<_> = snakes.into_iter().map(|v| {
        let mut value = v.split_whitespace();
        let fn_part = value.next().expect("at least one part");
        let operator_part = value.next();
        let name_string = if is_none {
            fn_part.to_string()
        } else {
            format!("where_{}", fn_part)
        };
        let fn_name = Ident::new(&name_string, fn_name.span());
        let original_block_tokens: proc_macro2::TokenStream = quote! { #fn_block };
        let original_block_string = original_block_tokens.to_string();
        let pascaled = if let Some(op) = operator_part {
            op
        } else {
            &v.to_pascal_case()
        };
        let enum_replace = format!("{} :: {}", enum_name, variant_name);
        let enum_replace_with = format!("{} :: {}", enum_name, pascaled);
        let fn_block_string = original_block_string.replace(&enum_replace, &enum_replace_with);
        let new_block_tokens: proc_macro2::TokenStream = fn_block_string
            .parse()
            .expect("Failed to re‐parse function body with Or replacement");

        let variant = if is_join {
            quote! {
                #[or_variant]
            }
        } else if is_none {
            quote! {
            }
        } else {
            quote! {
                #[condition_variant]
            }
        };
        let expanded = quote! {
            #variant
            pub #fn_unsafety #fn_asyncness fn #fn_name #fn_generics ( #fn_inputs ) #fn_output
            #fn_where
            {
                #new_block_tokens
            }
        };
        expanded
    })
        .collect();

    let expanded = quote! {
        #fn_vis #fn_unsafety #fn_asyncness fn #fn_name #fn_generics ( #fn_inputs ) #fn_output
        #fn_where
        {
            #fn_block
        }

        #(#fns)*
    };

    expanded.into()
}
