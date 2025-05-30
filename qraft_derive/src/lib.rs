use proc_macro::TokenStream;

mod binary;
mod unary;
mod between;
mod exists;

#[proc_macro_derive(BinaryOperator)]
pub fn operator_methods(input: TokenStream) -> TokenStream {
    binary::operator_methods_impl(input)
}

#[proc_macro_derive(UnaryOperator)]
pub fn unary_operator_methods(input: TokenStream) -> TokenStream {
    unary::unary_operator_methods_impl(input)
}

#[proc_macro_derive(BetweenOperator)]
pub fn between_operator_methods(input: TokenStream) -> TokenStream {
    between::between_operator_methods_impl(input)
}

//#[proc_macro_derive(InOperator)]
//pub fn in_operator_methods(input: TokenStream) -> TokenStream {
//    r#in::in_operator_methods_impl(input)
//}
//
#[proc_macro_derive(ExistsOperator)]
pub fn exists_operator_methods(input: TokenStream) -> TokenStream {
    exists::exists_operator_methods_impl(input)
}

//#[proc_macro_derive(Bindable, attributes(bindable))]
//pub fn bindable_derive(input: TokenStream) -> TokenStream {
//    bindable::bindable_derive_impl(input)
//}
