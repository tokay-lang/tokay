use proc_macro::TokenStream;
use proc_macro2;
use quote::{quote, quote_spanned};
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

fn gen_assign_arguments(
    required: Option<usize>,
    arguments: Vec<syn::Ident>,
) -> Vec<proc_macro2::TokenStream> {
    if required.is_some() {
        arguments
            .into_iter()
            .enumerate()
            .map(|(idx, arg)| {
                if idx < required.unwrap() {
                    quote! {
                        let mut #arg = args.get(#idx).unwrap().clone();
                    }
                } else {
                    quote! {
                        let mut #arg = args
                            .get(#idx)
                            .and_then(|x| Some(x.clone()))
                            .unwrap_or_else(|| crate::value::RefValue::from(Value::Void));
                    }
                }
            })
            .collect()
    } else {
        vec![quote! {
            assert!(args.is_empty());
        }]
    }
}

#[proc_macro]
pub fn tokay_method(input: TokenStream) -> TokenStream {
    let def = syn::parse_macro_input!(input as BuiltinDef);

    let name = def.function;
    let callable = syn::Ident::new(
        &format!("tokay_method_{}", name.to_string()),
        proc_macro2::Span::call_site(),
    );

    // Method names must start with a lower-case letter
    if !name.to_string().chars().next().unwrap().is_lowercase() {
        return quote_spanned! {
            name.span() => compile_error!(
                "Method identifier must start with a lower-case letter"
            );
        }
        .into();
    }

    // Generate assignment to identifier for each argument.
    let arguments = gen_assign_arguments(def.required, def.arguments);
    let body = def.body;

    // Generate two functions: One for direct usage from other Rust code,
    // and one wrapping function for calls from the Tokay VM or a Method.
    // The direct usage function will return an Result<RefValue, String>
    // instead of an Result<Accept, Reject>.
    let gen = quote! {
        pub fn #name(
            args: Vec<crate::value::RefValue>
        ) -> Result<crate::value::RefValue, String> {
            // The function's original name in Tokay
            let __function = stringify!(#name());

            // Arguments
            #(#arguments)*

            // Body
            #body
        }

        pub fn #callable(
            _context: Option<&mut crate::vm::Context>,
            args: Vec<crate::value::RefValue>
        ) -> Result<crate::vm::Accept, crate::vm::Reject> {
            let ret = Self::#name(args)?;
            Ok(crate::vm::Accept::Push(crate::vm::Capture::Value(ret, None, 10)))
        }
    };

    //println!("{} {:?}", function.to_string(), def.required);

    TokenStream::from(gen)
}

#[proc_macro]
pub fn tokay_function(input: TokenStream) -> TokenStream {
    let def = syn::parse_macro_input!(input as BuiltinDef);

    let name = def.function;
    let callable = syn::Ident::new(
        &format!("tokay_function_{}", name.to_string()),
        proc_macro2::Span::call_site(),
    );

    // Function names must start with a lower-case letter
    if !name.to_string().chars().next().unwrap().is_lowercase() {
        return quote_spanned! {
            name.span() => compile_error!(
                "Function identifier must start with a lower-case letter"
            );
        }
        .into();
    }

    // Generate assignment to identifier for each argument.
    let arguments = gen_assign_arguments(def.required, def.arguments);
    let body = def.body;

    // Generate function
    let gen = quote! {
        pub fn #callable(
            context: Option<&mut crate::vm::Context>,
            args: Vec<crate::value::RefValue>
        ) -> Result<crate::vm::Accept, crate::vm::Reject> {
            // The function's original name in Tokay
            let __function = stringify!(#name());

            // Arguments
            #(#arguments)*

            // Body
            #body
        }
    };

    TokenStream::from(gen)
}

#[proc_macro]
pub fn tokay_token(input: TokenStream) -> TokenStream {
    let def = syn::parse_macro_input!(input as BuiltinDef);

    let name = def.function;

    // Token names must start with an upper-case letter or underscore
    if !{
        let ch = name.to_string().chars().next().unwrap();
        ch.is_uppercase() || ch == '_'
    } {
        return quote_spanned! {
            name.span() => compile_error!(
                "Token identifier must start with an upper-case letter or underscore"
            );
        }
        .into();
    }

    let function = syn::Ident::new(
        &name.to_string().to_lowercase(),
        proc_macro2::Span::call_site(),
    );
    let callable = syn::Ident::new(
        &format!("tokay_token_{}", name.to_string().to_lowercase()),
        proc_macro2::Span::call_site(),
    );

    // Generate assignment to identifier for each argument.
    let arguments = gen_assign_arguments(def.required, def.arguments);
    let body = def.body;

    // Generate function and wrapper
    let gen = quote! {
        pub fn #function(
            context: &mut crate::vm::Context,
            args: Vec<crate::value::RefValue>
        ) -> Result<crate::vm::Accept, crate::vm::Reject> {
            // The function's original name in Tokay
            let __function = stringify!(#name());

            // Arguments
            #(#arguments)*

            // Body
            #body
        }

        pub fn #callable(
            context: Option<&mut crate::vm::Context>,
            args: Vec<crate::value::RefValue>
        ) -> Result<crate::vm::Accept, crate::vm::Reject> {
            #function(context.unwrap(), args)
        }
    };

    TokenStream::from(gen)
}
