use std::collections::VecDeque;

use heck::ToPascalCase;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, Ident};

#[proc_macro_attribute]
pub fn or_variant(_attr: TokenStream, item: TokenStream) -> TokenStream {
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
    let or_fn_name = Ident::new(&or_name_string, fn_name.span());

    let original_block_tokens: proc_macro2::TokenStream = quote! { #fn_block };
    let original_block_string = original_block_tokens.to_string();
    let or_block_string = original_block_string.replace("Conjunction :: And", "Conjunction :: Or");
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
pub fn condition_variant(_attr: TokenStream, item: TokenStream) -> TokenStream {
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
    let having_block_string = original_block_string.replace("where", "having");
    let having_block_tokens: proc_macro2::TokenStream = having_block_string
        .parse()
        .expect("Failed to re‐parse function body with Or replacement");

    let expanded = quote! {
        #[or_variant]
        #fn_vis #fn_unsafety #fn_asyncness fn #fn_name #fn_generics ( #fn_inputs ) #fn_output
        #fn_where
        {
            #fn_block
        }

        #[or_variant]
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
    let variant_name = snakes.pop_front().expect("expect the enum variant name");

    if snakes.len() < 2 {
        panic!("expected at least two operator inside #[binary(...)]");
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
        let name_string = format!("where_{}", fn_part);
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
        let expanded = quote! {
            #[condition_variant]
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
