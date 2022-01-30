use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, ItemFn, NestedMeta};

fn get_ident(attr: &[NestedMeta], item: &ItemFn) {
    let ident = item.sig.ident.to_string();
    println!("-- {} --", ident);

    for (i, att) in attr.iter().enumerate() {
        println!("{} {}: {:#?}" , ident, i, att)
    }
}

#[proc_macro_attribute]
pub fn tokay_method(args: TokenStream, input: TokenStream) -> TokenStream {
    println!("tokay_method");

    let attr = parse_macro_input!(args as AttributeArgs);
    let item = parse_macro_input!(input as ItemFn);

    get_ident(&attr, &item);

    TokenStream::from(quote!(#item))
}
