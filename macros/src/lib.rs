use proc_macro::TokenStream;
use proc_macro2;
use quote::quote;
use syn;

struct BuiltinDef {
    function: syn::Ident,
    arguments: Vec<syn::Ident>,
    body: syn::Block,
}

impl syn::parse::Parse for BuiltinDef {
    fn parse(stream: syn::parse::ParseStream) -> syn::Result<Self> {
        let function: syn::Ident = stream.parse()?;
        let content;
        let _: syn::token::Paren = syn::parenthesized!(content in stream);
        let arguments: syn::punctuated::Punctuated<syn::Ident, syn::Token![,]> =
            content.parse_terminated(syn::Ident::parse)?;
        let body: syn::Block = stream.parse()?;

        Ok(BuiltinDef {
            function,
            arguments: arguments.into_iter().collect(),
            body,
        })
    }
}

#[proc_macro]
pub fn tokay_method(input: TokenStream) -> TokenStream {
    let def = syn::parse_macro_input!(input as BuiltinDef);

    let function = def.function;
    let callable = syn::Ident::new(
        &format!("tokay_method_{}", function.to_string()),
        proc_macro2::Span::call_site(),
    );

    let arguments: Vec<proc_macro2::TokenStream> = def
        .arguments
        .into_iter()
        .enumerate()
        .map(|(idx, arg)| {
            quote! {
                let mut #arg = args.get(#idx).unwrap().clone();
            }
        })
        .collect();

    let body = def.body;

    let gen = quote! {
        pub fn #function(args: Vec<RefValue>) -> Result<RefValue, String> {
            #(#arguments)*
            #body
        }

        pub fn #callable(_context: Option<&mut Context>, args: Vec<RefValue>) -> Result<Accept, Reject> {
            let ret = Self::#function(args)?;
            Ok(Accept::Push(Capture::Value(ret, None, 10)))
        }
    };

    TokenStream::from(gen)
}
