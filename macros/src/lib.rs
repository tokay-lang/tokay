use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn;

struct BuiltinDef {
    function: syn::Ident,
    arguments: syn::LitStr,
    body: syn::Block,
}

impl syn::parse::Parse for BuiltinDef {
    fn parse(stream: syn::parse::ParseStream) -> syn::Result<Self> {
        let function: syn::Ident = stream.parse()?;
        let _comma: syn::token::Comma = stream.parse()?;
        let arguments: syn::LitStr = stream.parse()?;
        let _comma: syn::token::Comma = stream.parse()?;
        let body: syn::Block = stream.parse()?;

        Ok(BuiltinDef {
            function,
            arguments,
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
        Span::call_site(),
    );
    let body = def.body;

    let gen = quote! {
        pub fn #function(mut args: Vec<RefValue>) -> Result<RefValue, String> {
            #body
        }

        pub fn #callable(_context: Option<&mut Context>, args: Vec<RefValue>) -> Result<Accept, Reject> {
            let ret = Self::#function(args)?;
            Ok(Accept::Push(Capture::Value(ret, None, 10)))
        }
    };

    //println!("{:#?}", gen);

    TokenStream::from(gen)
}
