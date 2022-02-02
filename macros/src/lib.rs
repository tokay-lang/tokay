use proc_macro::TokenStream;
use proc_macro2;
use quote::quote;
use syn;

/// Allows to specify either an identifier or an required-delimiter `?` as parameter list.
enum BuiltinArgDef {
    Argument(syn::Ident),
    OptionalMarker(syn::Token![?]),
}

impl syn::parse::Parse for BuiltinArgDef {
    fn parse(stream: syn::parse::ParseStream) -> syn::Result<Self> {
        if stream.peek(syn::Token![?]) {
            Ok(BuiltinArgDef::OptionalMarker(stream.parse()?))
        } else {
            Ok(BuiltinArgDef::Argument(stream.parse()?))
        }
    }
}

/// Describes a builtin function and its arguments.
struct BuiltinDef {
    function: syn::Ident,
    arguments: Vec<syn::Ident>,
    required: Option<usize>,
    body: syn::Block,
}

impl syn::parse::Parse for BuiltinDef {
    fn parse(stream: syn::parse::ParseStream) -> syn::Result<Self> {
        let function: syn::Ident = stream.parse()?;
        //let function = stream.parse::<syn::Ident>()?;
        let content;
        let _: syn::token::Paren = syn::parenthesized!(content in stream);
        let argdefs: syn::punctuated::Punctuated<BuiltinArgDef, syn::Token![,]> =
            content.parse_terminated(BuiltinArgDef::parse)?;
        let body: syn::Block = stream.parse()?;

        // Collect arguments and possible required marker.
        let mut arguments = Vec::new();
        let mut required = None;

        for (i, argdef) in argdefs.into_iter().enumerate() {
            match argdef {
                BuiltinArgDef::Argument(arg) => arguments.push(arg),
                BuiltinArgDef::OptionalMarker(marker) => {
                    if required.is_some() {
                        return Err(syn::parse::Error::new(
                            marker.span,
                            "Cannot provide multiple required delimiters",
                        ));
                    }

                    required = Some(i)
                }
            }
        }

        // If no required-marker was set but arguments where defined,
        // all arguments are required.
        if required.is_none() && !arguments.is_empty() {
            required = Some(arguments.len());
        }

        Ok(BuiltinDef {
            function,
            arguments,
            required,
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

    // Generate assignment to identifier for each argument.
    let arguments: Vec<proc_macro2::TokenStream> = if def.required.is_some() {
        def
        .arguments
        .into_iter()
        .enumerate()
        .map(|(idx, arg)| {
            if idx < def.required.unwrap() {
                quote! {
                    let mut #arg = args.get(#idx).unwrap().clone();
                }
            }
            else {
                quote! {
                    let mut #arg = args
                        .get(#idx)
                        .and_then(|x| Some(x.clone()))
                        .unwrap_or_else(|| crate::value::RefValue::from(Value::Void));
                }
            }
        })
        .collect()
    }
    else {
        vec![
            quote! {
                assert!(args.is_empty());
            }
        ]
    };

    let body = def.body;

    // Generate two functions: One for direct usage from other Rust code,
    // and one wrapping function for calls from the Tokay VM or a Method.
    let gen = quote! {
        pub fn #function(args: Vec<crate::value::RefValue>) -> Result<crate::value::RefValue, String> {
            #(#arguments)*
            #body
        }

        pub fn #callable(
            _context: Option<&mut crate::vm::Context>,
            args: Vec<crate::value::RefValue>
        ) -> Result<crate::vm::Accept, crate::vm::Reject> {
            let ret = Self::#function(args)?;
            Ok(crate::vm::Accept::Push(crate::vm::Capture::Value(ret, None, 10)))
        }
    };

    //println!("{} {:?}", function.to_string(), def.required);

    TokenStream::from(gen)
}
