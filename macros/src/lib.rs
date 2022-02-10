/*! Tokay proc-macros

This crate contains the proc-macro implementation for

- tokay_function!() - Built-in function
- tokay_method!() - Built-in object method
- tokay_token!() - Built-in consuming function

Every macro generates a slightly different version of a callable built-in.

All macros require for two parameters:

- *signature* is a Tokay-style function signature string, including default values.
  This can be `f`, `f()`, `f(a, b)`, `f(a b = void)` or similar.
  Currently, this does only accept for a subset of Tokay atomics: void, null, true, false.
- *expression* is the Rust expression to be executed. This is the body of the function.
*/

use proc_macro::TokenStream;
use proc_macro2;
use quote::{quote, quote_spanned};
use syn;
use tokay;

/* Tokay v0.4 compat, the function must be reworked in v0.5 */
fn tokay_run(src: &str, input: &str) -> Result<Option<tokay::value::Value>, String> {
    let mut compiler = tokay::compiler::Compiler::new();
    let program = compiler.compile(tokay::reader::Reader::new(Box::new(std::io::Cursor::new(
        src.to_owned(),
    ))));

    match program {
        Ok(program) => program
            .run_from_string(input.to_owned())
            .map_err(|err| err.to_string()),
        Err(errors) => Err(errors
            .into_iter()
            .map(|err| err.to_string())
            .collect::<Vec<String>>()
            .join("\n")),
    }
}

/// Describes a builtin function and its arguments.
struct BuiltinDef {
    name: syn::Ident,
    arguments: Vec<(syn::Ident, String)>,
    body: syn::Expr,
}

impl syn::parse::Parse for BuiltinDef {
    fn parse(stream: syn::parse::ParseStream) -> syn::Result<Self> {
        let signature = stream.parse::<syn::LitStr>()?;
        let _ = stream.parse::<syn::Token![,]>()?;
        let body = stream.parse::<syn::Expr>()?;

        // Collect arguments and possible required marker.
        let res = match tokay_run(include_str!("signature.tok"), &signature.value()) {
            Err(msg) => return Err(syn::parse::Error::new(signature.span(), msg)),
            Ok(ast) => ast.unwrap().to_list(),
        };

        let mut arguments = Vec::new();
        let name = res[0].borrow().to_string();

        if res.len() > 1 {
            let args = res[1].borrow().to_list();

            // fixme: This is a little bit ugly but is needed to use Tokay v0.4 here.
            //        Is has to be improved when using a higher Tokay version for building later.
            for item in args.iter() {
                let arg = &*item.borrow();
                if let Some(arg) = arg.get_list() {
                    //println!("{} {:?}", name, item);
                    arguments.push((
                        syn::Ident::new(
                            arg[0].borrow().get_string().unwrap(),
                            proc_macro2::Span::call_site(),
                        ),
                        arg[1].borrow().to_string(),
                    ));
                } else {
                    //println!("{} {:?}", name, args);
                    arguments.push((
                        syn::Ident::new(
                            args[0].borrow().get_string().unwrap(),
                            proc_macro2::Span::call_site(),
                        ),
                        args[1].borrow().to_string(),
                    ));
                    break; // Tokay v0.4 special case... don't ask for this.
                }
            }
        }

        Ok(BuiltinDef {
            name: syn::Ident::new(&name, proc_macro2::Span::call_site()),
            arguments,
            body,
        })
    }
}

fn gen_assign_arguments(arguments: Vec<(syn::Ident, String)>) -> Vec<proc_macro2::TokenStream> {
    if arguments.len() > 0 {
        arguments
            .into_iter()
            .enumerate()
            .map(|(idx, (arg, default))| {
                //println!("{} => {} = {}", idx, arg, default);

                if default.is_empty() {
                    quote! {
                        let mut #arg = args.get(#idx).unwrap().clone();
                    }
                } else {
                    let default = match &default[..] {
                        "void" => stringify!(Value::Void),
                        "null" => stringify!(Value::Null),
                        "true" => stringify!(Value::True),
                        "false" => stringify!(Value::False),
                        _ => unreachable!(),
                    };

                    quote! {
                        let mut #arg = args
                            .get(#idx)
                            .and_then(|x| Some(x.clone()))
                            .unwrap_or_else(|| crate::value::RefValue::from(#default));
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

    let name = def.name;
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
    let arguments = gen_assign_arguments(def.arguments);
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

    let name = def.name;
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
    let arguments = gen_assign_arguments(def.arguments);
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

    let name = def.name;

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
    let arguments = gen_assign_arguments(def.arguments);
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
