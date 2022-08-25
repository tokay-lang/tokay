//! Compiler's internal Abstract Syntax Tree traversal
use std::collections::HashSet;
use tokay_macros::tokay_function;
extern crate self as tokay;
use super::*;
use crate::error::Error;
use crate::reader::Offset;
use crate::utils;
use crate::value;
use crate::value::{Dict, List, Object, RefValue, Str, Token};
use crate::vm::*;
use charclass::CharClass;

/// Checks whether identifier's name is the name of a reserved word.
fn identifier_is_valid(ident: &str) -> Result<(), Error> {
    match ident {
        "accept" | "begin" | "break" | "continue" | "else" | "end" | "exit" | "expect"
        | "false" | "for" | "if" | "in" | "loop" | "next" | "not" | "null" | "peek" | "push"
        | "reject" | "repeat" | "return" | "true" | "void" => Err(Error::new(
            None,
            format!("Expected identifier, found reserved word '{}'", ident),
        )),
        _ => Ok(()),
    }
}

/// AST traversal entry
pub(super) fn traverse(compiler: &mut Compiler, ast: &RefValue) -> ImlOp {
    traverse_node_or_list(compiler, ast)
}

// Traverse either a node or a list from the AST
fn traverse_node_or_list(compiler: &mut Compiler, ast: &RefValue) -> ImlOp {
    if let Some(list) = ast.borrow().object::<List>() {
        let mut ops = Vec::new();

        for item in list.iter() {
            ops.push(traverse_node_or_list(compiler, item));
        }

        ImlOp::from(ops)
    } else if let Some(dict) = ast.borrow().object::<Dict>() {
        traverse_node(compiler, dict)
    } else {
        ImlOp::load(None, ImlValue::from(RefValue::from(ast.clone())))
    }
}

// Extract offset positions into an Offset structure
fn traverse_node_offset(node: &Dict) -> Option<Offset> {
    let offset = node
        .get("offset")
        .and_then(|offset| Some(offset.to_usize().unwrap()));
    let row = node
        .get("row")
        .and_then(|row| Some(row.to_usize().unwrap() as u32));
    let col = node
        .get("col")
        .and_then(|col| Some(col.to_usize().unwrap() as u32));

    if let (Some(offset), Some(row), Some(col)) = (offset, row, col) {
        Some(Offset { offset, row, col })
    } else {
        None
    }
}

// Append offset to ops
fn traverse_offset(node: &Dict) -> ImlOp {
    if let Some(offset) = traverse_node_offset(node) {
        ImlOp::from(Op::Offset(Box::new(offset)))
    } else {
        ImlOp::Nop
    }
}

// Traverse a value node into an ImlValue instance
fn traverse_node_value(compiler: &mut Compiler, node: &Dict) -> ImlValue {
    let emit = node["emit"].borrow();
    let emit = emit.object::<Str>().unwrap().as_str();

    // Generate a value from the given code
    match emit {
        // Literals
        "value_string" => ImlValue::from(node["value"].clone()),
        "value_integer" => node["value"].clone().into(),
        "value_float" => node["value"].clone().into(),
        "value_true" => value!(true).into(),
        "value_false" => value!(false).into(),
        "value_null" => value!(null).into(),
        "value_void" => value!(void).into(),

        // Tokens
        "value_token_match" | "value_token_touch" => {
            let mut value = node["value"].to_string();

            if value.len() == 0 {
                compiler.errors.push(Error::new(
                    traverse_node_offset(node),
                    format!("Empty match not allowed"),
                ));
                value = "#INVALID".to_string();
            }

            if emit == "value_token_match" {
                RefValue::from(Token::Match(value)).into()
            } else {
                RefValue::from(Token::Touch(value)).into()
            }
        }
        "value_token_any" => RefValue::from(Token::any()).into(),
        "value_token_ccl" => {
            let node = node["children"].borrow();
            let node = node.object::<Dict>().unwrap();

            let emit = node["emit"].borrow();
            let emit = emit.object::<Str>().unwrap().as_str();

            let children = List::from(&node["children"]);

            let mut ccl = CharClass::new();

            for range in children.iter() {
                let range = range.borrow();
                let range = range.object::<Dict>().unwrap();

                let emit = range["emit"].borrow();
                let emit = emit.object::<Str>().unwrap().as_str();

                let value = range["value"].borrow();
                let value = value.object::<Str>().unwrap().as_str();

                match &emit[..] {
                    "char" => {
                        let ch = value.chars().next().unwrap();
                        ccl.add(ch..=ch);
                    }
                    "range" => {
                        let from = value.chars().nth(0).unwrap();
                        let to = value.chars().nth(1).unwrap();

                        ccl.add(from..=to);
                    }
                    _ => {
                        unreachable!();
                    }
                }
            }

            if emit == "ccl_neg" {
                RefValue::from(Token::Char(ccl.negate())).into()
            } else {
                assert!(emit == "ccl");
                RefValue::from(Token::Char(ccl)).into()
            }
        }

        // Parselets
        "value_parselet" => {
            compiler.push_parselet();

            // Generic signature
            let mut gen: Vec<(String, Option<ImlValue>)> = Vec::new();
            let mut gen_names = HashSet::new();

            // Function signature
            let mut sig: Vec<(String, Option<ImlValue>)> = Vec::new();
            let mut sig_names = HashSet::new();

            // Traverse the AST
            let mut sigs = List::from(node["children"].clone());
            let body = sigs.pop().unwrap();

            for node in sigs.into_iter() {
                let node = node.borrow();
                let node = node.object::<Dict>().unwrap();

                let emit = node["emit"].borrow();
                let emit = emit.object::<Str>().unwrap().as_str();

                let children = List::from(node["children"].clone());
                let ident = children[0].borrow().object::<Dict>().unwrap()["value"].to_string();

                match emit {
                    "gen" => {
                        // check if identifier was not provided twice
                        if gen_names.contains(&ident) {
                            compiler.errors.push(Error::new(
                                traverse_node_offset(node),
                                format!("Generic '{}' already given in signature before", ident),
                            ));

                            continue;
                        } else {
                            gen_names.insert(ident.clone());
                        }

                        compiler.set_constant(&ident, ImlValue::Undetermined(ident.to_string()));

                        assert!(children.len() <= 2);
                        let default = if children.len() == 2 {
                            let default = children[1].borrow();
                            let value = traverse_node_static(
                                compiler,
                                Some(&ident),
                                default.object::<Dict>().unwrap(),
                            );
                            Some(value)
                        } else {
                            None
                        };

                        gen.push((ident.to_string(), default));
                        //println!("{} {} {:?}", emit.to_string(), ident, default);
                    }
                    "arg" => {
                        let first = ident.chars().nth(0).unwrap();

                        // Check for correct identifier semantics
                        if !first.is_lowercase() {
                            compiler.errors.push(
                                Error::new(
                                    traverse_node_offset(node),
                                    if first == '_' {
                                        format!(
                                            "Argument named '{}' invalid; May not start with '{}'",
                                            ident, first
                                        )
                                    }
                                    else {
                                        format!(
                                            "Argument named '{}' invalid; Use a name starting in lower-case, e.g. '{}{}'",
                                            ident, &ident[0..1].to_lowercase(), &ident[1..]
                                        )
                                    }
                                )
                            );
                        }

                        // check if identifier was not provided twice
                        if sig_names.contains(&ident) {
                            compiler.errors.push(Error::new(
                                traverse_node_offset(node),
                                format!("Argument '{}' already given in signature before", ident),
                            ));

                            continue;
                        } else {
                            sig_names.insert(ident.clone());
                        }

                        compiler.new_local(&ident);

                        assert!(children.len() <= 2);
                        let default = if children.len() == 2 {
                            let default = children[1].borrow();
                            let value = traverse_node_static(
                                compiler,
                                Some(&ident),
                                default.object::<Dict>().unwrap(),
                            );
                            Some(value)
                        } else {
                            None
                        };

                        sig.push((ident.to_string(), default));
                        //println!("{} {} {:?}", emit.to_string(), ident, default);
                    }
                    _ => unreachable!(),
                }
            }

            let body = traverse_node(compiler, body.borrow().object::<Dict>().unwrap());

            let ret = compiler.pop_parselet(
                traverse_node_offset(node),
                None,
                None,
                Some(gen),
                Some(sig),
                body,
            );

            //println!("parselet = {:#?}", ret);
            return ret;
        }
        _ => unimplemented!("unhandled value node {}", emit),
    }
}

/** Traverse a static value.
The value must either be a literal or something from a known constant.

The name attribute is optional and can be used to assign an identifier to parselets for debug purposes
*/
fn traverse_node_static(compiler: &mut Compiler, lvalue: Option<&str>, node: &Dict) -> ImlValue {
    compiler.push_parselet(); // yep, we push a parselet scope here...

    match traverse_node_rvalue(compiler, node, Rvalue::Load) {
        ImlOp::Nop => {
            compiler.pop_parselet(None, None, None, None, None, ImlOp::Nop);
            value!(void).into()
        }
        // Defined parselet or value
        ImlOp::Load {
            target: ImlTarget::Static(value),
            ..
        } => {
            compiler.pop_parselet(None, None, None, None, None, ImlOp::Nop);

            if let Some(lvalue) = lvalue {
                if let ImlValue::Parselet(parselet) = &value {
                    let mut parselet = parselet.borrow_mut();
                    parselet.name = Some(lvalue.to_string());
                }
            }

            value
        }
        // Any other code becomes its own parselet without any signature.
        other => compiler.pop_parselet(
            traverse_node_offset(node),
            lvalue.and_then(|lvalue| Some(lvalue.to_string())),
            None,
            None,
            None,
            other,
        ),
    }
}

// Traverse lvalue
fn traverse_node_lvalue(compiler: &mut Compiler, node: &Dict, store: bool, hold: bool) -> ImlOp {
    let emit = node["emit"].borrow();
    let emit = emit.object::<Str>().unwrap().as_str();
    assert!(emit == "lvalue");

    let children = List::from(&node["children"]);

    let mut ops: Vec<ImlOp> = Vec::new();

    for (i, item) in children.iter().enumerate() {
        let item = item.borrow();
        let item = item.object::<Dict>().unwrap();

        let emit = item["emit"].borrow();
        let emit = emit.object::<Str>().unwrap().as_str();

        let store = if i < children.len() - 1 { false } else { store };

        match emit {
            capture if capture.starts_with("capture") => {
                let children = item["children"].borrow();

                match capture {
                    "capture_expr" | "capture_alias" => {
                        ops.push(traverse_node_or_list(compiler, &item["children"]));

                        if store {
                            if hold {
                                ops.push(Op::StoreCaptureHold.into())
                            } else {
                                ops.push(Op::StoreCapture.into())
                            }
                        } else {
                            ops.push(Op::LoadCapture.into())
                        }
                    }

                    "capture_index" => {
                        let children = children.object::<Dict>().unwrap();
                        let index = traverse_node_value(compiler, children).value();

                        if store {
                            if hold {
                                ops.push(
                                    Op::StoreFastCaptureHold(index.to_usize().unwrap()).into(),
                                );
                            } else {
                                ops.push(Op::StoreFastCapture(index.to_usize().unwrap()).into());
                            }
                        } else {
                            ops.push(Op::LoadFastCapture(index.to_usize().unwrap()).into());
                        }
                    }

                    _ => {
                        unreachable!();
                    }
                }
            }

            "identifier" => {
                let name = item["value"].borrow();
                let name = name.object::<Str>().unwrap().as_str();

                // Check for not assigning to a constant (at any level)
                if compiler.get_constant(name).is_some() {
                    compiler.errors.push(Error::new(
                        traverse_node_offset(node),
                        format!("Cannot assign to constant '{}'", name),
                    ));

                    break;
                }

                // Check if identifier is valid
                if let Err(mut error) = identifier_is_valid(name) {
                    if let Some(offset) = traverse_node_offset(node) {
                        error.patch_offset(offset);
                    }

                    compiler.errors.push(error);
                    break;
                }

                // Check if identifier is not defining a consumable
                if utils::identifier_is_consumable(name) {
                    compiler.errors.push(Error::new(
                        traverse_node_offset(node),

                        if &name[0..1] == "_" {
                            format!(
                                "The variable '{}' is invalid, only constants may start with '_'",
                                name
                            )
                        }
                        else {
                            format!(
                                "Cannot assign variable named '{}'; Try lower-case identifier, e.g. '{}'",
                                name, name.to_lowercase()
                            )
                        }
                    ));

                    break;
                }

                /* Generates code for a symbol store, which means:

                    1. look-up local variable, and store into
                    2. look-up global variable, and store into
                    3. create local variable, and store into
                */
                if let Some(addr) = compiler.get_local(name) {
                    if store {
                        if hold {
                            ops.push(Op::StoreFastHold(addr).into())
                        } else {
                            ops.push(Op::StoreFast(addr).into())
                        }
                    } else {
                        ops.push(Op::LoadFast(addr).into())
                    }
                } else if let Some(addr) = compiler.get_global(name) {
                    if store {
                        if hold {
                            ops.push(Op::StoreGlobalHold(addr).into())
                        } else {
                            ops.push(Op::StoreGlobal(addr).into())
                        }
                    } else {
                        ops.push(Op::LoadGlobal(addr).into())
                    }
                } else {
                    let addr = compiler.new_local(name);
                    if store {
                        if hold {
                            ops.push(Op::StoreFastHold(addr).into())
                        } else {
                            ops.push(Op::StoreFast(addr).into())
                        }
                    } else {
                        ops.push(Op::LoadFast(addr).into())
                    }
                }
            }

            // index ----------------------------------------------------------
            "index" => {
                ops.push(traverse_node_or_list(compiler, &item["children"]));

                if store {
                    if hold {
                        ops.push(Op::StoreIndexHold.into()); // todo: in case value is an integer, use LoadFastIndexHold
                    } else {
                        ops.push(Op::StoreIndex.into()); // todo: in case value is an integer, use LoadFastIndex
                    }
                } else {
                    ops.push(Op::LoadIndex.into())
                }
            }

            other => {
                unimplemented!("{:?} not implemented for lvalue", other);
            }
        }
    }

    ImlOp::from(ops)
}

/** Enum used to specify traverse_node_rvalue code generation.

Rvalue::Load generates code to just load the value,
Rvalue::CallOrLoad generates code to either call the value without parameters or load it
Rvalue::Call(args, nargs) generates code for a full-qualified value call
*/
enum Rvalue {
    Load,              // Generate code to just load the value
    CallOrLoad,        // Generate code for a call without parameters, or load otherwise
    Call(usize, bool), // Generate code for a full qualified call
}

// Traverse an rvalue
fn traverse_node_rvalue(compiler: &mut Compiler, node: &Dict, mode: Rvalue) -> ImlOp {
    let emit = node["emit"].borrow();
    let emit = emit.object::<Str>().unwrap().as_str();

    let ops = match emit {
        // attribute ------------------------------------------------------
        "attribute" => ImlOp::from(vec![
            {
                let children = node["children"].borrow();
                traverse_node_rvalue(
                    compiler,
                    children.object::<Dict>().unwrap(),
                    Rvalue::CallOrLoad,
                )
            },
            ImlOp::from(Op::LoadAttr),
        ]),

        // rvalue ---------------------------------------------------------
        "rvalue" => {
            let children = List::from(&node["children"]);

            let mut ops = vec![traverse_offset(node)];

            for node in children.iter() {
                ops.push(traverse_node_rvalue(
                    compiler,
                    node.borrow().object::<Dict>().unwrap(),
                    Rvalue::Load,
                ));
            }

            assert!(ops.len() > 0);
            ImlOp::from(ops)
        }

        // call -----------------------------------------------------------
        "call" => {
            let mut ops = Vec::new();
            let mut args = 0;
            let mut nargs = 0;

            // Traverse call parameters from AST
            let children = List::from(&node["children"]);

            for param in &children[1..] {
                let param = param.borrow();
                let param = param.object::<Dict>().unwrap();

                let emit = param["emit"].borrow();

                match emit.object::<Str>().unwrap().as_str() {
                    "param" => {
                        if nargs > 0 {
                            compiler.errors.push(Error::new(
                                traverse_node_offset(node),
                                format!(
                                    "Sequencial arguments need to be specified before named arguments."
                                ),
                            ));

                            continue;
                        }

                        let param = &param["children"].borrow();
                        let param = param.object::<Dict>().unwrap();

                        ops.push(traverse_node_rvalue(compiler, param, Rvalue::CallOrLoad));
                        args += 1;
                    }

                    "param_named" => {
                        let children = List::from(&param["children"]);

                        let param = &children[1].borrow();
                        let param = param.object::<Dict>().unwrap();

                        ops.push(traverse_node_rvalue(compiler, param, Rvalue::CallOrLoad));

                        let ident = children[0].borrow();
                        let ident = ident.object::<Dict>().unwrap();
                        let ident = ident["value"].borrow();
                        let ident = ident.object::<Str>().unwrap().as_str();

                        ops.push(ImlOp::load(
                            traverse_node_offset(&param),
                            ImlValue::from(RefValue::from(ident)),
                        ));

                        nargs += 1;
                    }

                    other => unimplemented!("Unhandled parameter type {:?}", other),
                }
            }

            // When calling with nargs, create a nargs dict first
            if nargs > 0 {
                ops.push(ImlOp::from(Op::MakeDict(nargs)));
            }

            let rvalue = children[0].borrow();
            ops.push(traverse_node_rvalue(
                compiler,
                rvalue.object::<Dict>().unwrap(),
                Rvalue::Call(args, nargs > 0),
            ));

            ImlOp::from(ops)
        }

        // capture --------------------------------------------------------
        "capture_alias" | "capture_expr" => ImlOp::from(vec![
            {
                let children = node["children"].borrow();
                traverse_node_rvalue(
                    compiler,
                    children.object::<Dict>().unwrap(),
                    Rvalue::CallOrLoad,
                )
            },
            ImlOp::from(Op::LoadCapture),
        ]),

        "capture_index" => {
            let children = node["children"].borrow();
            let children = children.object::<Dict>().unwrap();
            let index = traverse_node_value(compiler, children).value();
            ImlOp::from(Op::LoadFastCapture(index.to_usize().unwrap()))
        }

        // identifier -----------------------------------------------------
        "identifier" => {
            let name = node["value"].borrow();
            let name = name.object::<Str>().unwrap().as_str();

            let offset = traverse_node_offset(node);

            // Check if identifier is valid
            return if let Err(mut error) = identifier_is_valid(name) {
                if let Some(offset) = offset {
                    error.patch_offset(offset);
                }

                compiler.errors.push(error);
                ImlOp::Nop
            } else {
                match mode {
                    Rvalue::Load => ImlOp::load_by_name(compiler, offset, name.to_string()),
                    Rvalue::CallOrLoad => {
                        // fixme: this detection might be improved by ImlOp::consumable()
                        if utils::identifier_is_consumable(name) {
                            compiler.mark_consuming();
                        }

                        ImlOp::call_by_name(compiler, offset, name.to_string(), None)
                    }
                    Rvalue::Call(args, nargs) => {
                        // fixme: this detection might be improved by ImlOp::consumable()
                        if utils::identifier_is_consumable(name) {
                            compiler.mark_consuming();
                        }

                        ImlOp::call_by_name(compiler, offset, name.to_string(), Some((args, nargs)))
                    }
                }
            };
        }

        // index ----------------------------------------------------------
        "index" => {
            ImlOp::from(vec![
                traverse_node_or_list(compiler, &node["children"]),
                traverse_offset(node),
                ImlOp::from(Op::LoadIndex), // todo: in case value is an integer, use LoadFastIndex
            ])
        }

        // inplace --------------------------------------------------------
        inplace if inplace.starts_with("inplace_") => {
            let children = node["children"].borrow();
            let lvalue = children.object::<Dict>().unwrap();

            let mut ops = vec![
                traverse_node_lvalue(compiler, lvalue, false, false),
                traverse_offset(node),
            ];

            let parts: Vec<&str> = inplace.split("_").collect();

            match parts[1] {
                "pre" => {
                    ops.push(
                        if parts[2] == "inc" {
                            Op::UnaryOp("iinc")
                        } else {
                            Op::UnaryOp("idec")
                        }
                        .into(),
                    );
                }
                "post" => {
                    ops.extend(vec![
                        Op::Dup.into(),
                        Op::Rot2.into(),
                        if parts[2] == "inc" {
                            Op::UnaryOp("iinc")
                        } else {
                            Op::UnaryOp("idec")
                        }
                        .into(),
                        Op::Drop.into(),
                    ]);
                }
                _ => unreachable!(),
            }

            ImlOp::from(ops)
        }

        // value ---------------------------------------------------------
        value if value.starts_with("value_") => {
            let offset = traverse_node_offset(node);
            let value = traverse_node_value(compiler, node);

            return match mode {
                Rvalue::Load => ImlOp::load(offset, value),
                Rvalue::CallOrLoad => ImlOp::call(offset, value, None),
                Rvalue::Call(args, nargs) => ImlOp::call(offset, value, Some((args, nargs))),
            };
        }

        // anything else is not an rvalue
        _ => return traverse_node(compiler, node),
    };

    // Rvalue call confguration?
    match mode {
        Rvalue::Load => ops,
        Rvalue::CallOrLoad => ImlOp::from(vec![ops, ImlOp::from(Op::CallOrCopy)]),
        Rvalue::Call(args, nargs) => ImlOp::from(vec![
            ops,
            if args == 0 && !nargs {
                ImlOp::from(Op::Call)
            } else if args > 0 && !nargs {
                ImlOp::from(Op::CallArg(args))
            } else {
                ImlOp::from(Op::CallArgNamed(args))
            },
        ]),
    }
}

// Main traversal function, running recursively through the AST
fn traverse_node(compiler: &mut Compiler, node: &Dict) -> ImlOp {
    // Normal node processing...
    let emit = node["emit"].borrow();
    let emit = emit.object::<Str>().unwrap().as_str();

    //println!("emit = {:?}", emit);

    match emit {
        "alias" => {
            let children = node["children"].borrow();
            let children = children.object::<List>().unwrap();

            let (alias, expr) = (&children[0].borrow(), &children[1].borrow());

            let alias =
                traverse_node_rvalue(compiler, alias.object::<Dict>().unwrap(), Rvalue::Load);
            let expr =
                traverse_node_rvalue(compiler, expr.object::<Dict>().unwrap(), Rvalue::CallOrLoad);

            // Push value first, then the alias
            ImlOp::from(vec![expr, alias, ImlOp::from(Op::MakeAlias)])
        }

        // assign ---------------------------------------------------------
        assign if assign.starts_with("assign") => {
            let children = node["children"].borrow();
            let children = children.object::<List>().unwrap();

            let (lvalue, value) = (children[0].borrow(), children[1].borrow());
            let lvalue = lvalue.object::<Dict>().unwrap();
            let value = value.object::<Dict>().unwrap();

            let parts: Vec<&str> = assign.split("_").collect();

            let mut ops = Vec::new();

            if parts.len() > 1 && parts[1] != "hold" {
                ops.push(traverse_node_lvalue(compiler, lvalue, false, false));
                ops.push(traverse_node_rvalue(compiler, value, Rvalue::Load));

                ops.push(match parts[1] {
                    "add" => ImlOp::from(Op::BinaryOp("iadd")),
                    "sub" => ImlOp::from(Op::BinaryOp("isub")),
                    "mul" => ImlOp::from(Op::BinaryOp("imul")),
                    "div" => ImlOp::from(Op::BinaryOp("idiv")),
                    _ => unreachable!(),
                });

                if *parts.last().unwrap() != "hold" {
                    ops.push(Op::Drop.into());
                }
            } else {
                ops.push(traverse_node_rvalue(compiler, value, Rvalue::Load));
                ops.push(traverse_offset(node));
                ops.push(traverse_node_lvalue(
                    compiler,
                    lvalue,
                    true,
                    *parts.last().unwrap() == "hold",
                ));
            }

            ImlOp::from(ops)
        }

        // begin ----------------------------------------------------------
        "begin" | "end" => {
            let body = traverse_node_or_list(compiler, &node["children"]);

            if let Scope::Parselet { begin, end, .. } = &mut compiler.scopes[0] {
                if emit == "begin" {
                    begin.push(body)
                } else {
                    end.push(body)
                }
            } else {
                compiler.errors.push(Error::new(
                    traverse_node_offset(node),
                    format!("'{}' may only be used in parselet scope", emit),
                ));
            }

            ImlOp::Nop
        }

        // block ----------------------------------------------------------
        "block" | "main" => {
            if let Some(ast) = node.get("children") {
                if emit == "block" {
                    compiler.push_block();
                }
                // When interactive and there's a scope, don't push, as the main scope
                // is kept to hold globals.
                else if compiler.scopes.len() != 1 {
                    compiler.push_parselet(); // Main
                }

                let body = if let Some(list) = ast.borrow().object::<List>() {
                    let mut alts = Vec::new();

                    for item in list.iter() {
                        alts.push(traverse_node_or_list(compiler, item));
                    }

                    ImlOp::Alt { alts }
                } else if let Some(dict) = ast.borrow().object::<Dict>() {
                    traverse_node(compiler, dict)
                } else {
                    unreachable!();
                };

                if emit == "block" {
                    compiler.pop_block();
                    body
                } else {
                    let main = compiler.pop_parselet(
                        None,
                        Some("__main__".to_string()),
                        None,
                        None,
                        None,
                        body,
                    );

                    ImlOp::call(None, main, None)
                }
            } else {
                ImlOp::Nop
            }
        }

        // constant -------------------------------------------------------
        "constant" => {
            let children = node["children"].borrow();
            let children = children.object::<List>().unwrap();

            let (ident, value) = (children[0].borrow(), children[1].borrow());

            let ident = ident.object::<Dict>().unwrap();
            let ident = ident["value"].borrow();
            let ident = ident.object::<Str>().unwrap().as_str();

            if let Err(mut error) = identifier_is_valid(ident) {
                if let Some(offset) = traverse_node_offset(node) {
                    error.patch_offset(offset);
                }

                compiler.errors.push(error);
                return ImlOp::Nop;
            }

            // Distinguish between pure values or an expression
            let value = value.object::<Dict>().unwrap();

            // fixme: Restricted to pure values currently.
            let value = traverse_node_static(compiler, Some(&ident), value);

            if value.is_consuming() {
                if !utils::identifier_is_consumable(ident) {
                    compiler.errors.push(Error::new(
                        traverse_node_offset(node),
                        format!(
                            "Cannot assign constant '{}' as consumable. Use an identifier starting in upper-case, e.g. '{}{}'",
                            ident, &ident[0..1].to_uppercase(), &ident[1..]
                        )
                    ));

                    return ImlOp::Nop;
                }
            } else if utils::identifier_is_consumable(ident) {
                compiler.errors.push(Error::new(
                    traverse_node_offset(node),
                    if ident.starts_with("_") {
                        format!(
                            "Cannot assign to constant '{}', because it must be consumable. Use an identifier not starting with '_'.",
                            ident
                        )
                    }
                    else {
                        format!(
                            "Cannot assign to constant '{}', because it must be consumable. Use an identifier starting in lower-case, e.g. '{}{}'",
                            ident, &ident[0..1].to_lowercase(), &ident[1..]
                        )
                    }
                ));

                return ImlOp::Nop;
            }

            //println!("{} : {:?}", ident, value);
            compiler.set_constant(ident, value);

            // Try to resolve usage of newly introduced constant in current scope
            compiler.resolve();

            ImlOp::Nop
        }

        // operator ------------------------------------------------------
        op if op.starts_with("op_") => {
            let parts: Vec<&str> = emit.split("_").collect();
            let mut ops = Vec::new();

            let op = match parts[1] {
                "accept" | "break" | "exit" | "push" | "repeat" => {
                    if parts[1] == "break" && !compiler.check_loop() {
                        compiler.errors.push(Error::new(
                            traverse_node_offset(node),
                            format!("'break' cannot be used outside of a loop."),
                        ));
                    }

                    if let Some(value) = node.get("children") {
                        let value = value.borrow();
                        ops.push(traverse_node_rvalue(
                            compiler,
                            &value.object::<Dict>().unwrap(),
                            Rvalue::CallOrLoad,
                        ));

                        match parts[1] {
                            "accept" => Op::LoadAccept.into(),
                            "break" => Op::LoadBreak.into(),
                            "exit" => Op::LoadExit.into(),
                            "push" => Op::LoadPush.into(),
                            "repeat" => Op::LoadRepeat.into(),
                            _ => unreachable!(),
                        }
                    } else {
                        match parts[1] {
                            "accept" => Op::Accept.into(),
                            "break" => Op::Break.into(),
                            "exit" => Op::Exit.into(),
                            "push" => Op::Push.into(),
                            "repeat" => Op::Repeat.into(),
                            _ => unreachable!(),
                        }
                    }
                }

                "continue" => {
                    if !compiler.check_loop() {
                        compiler.errors.push(Error::new(
                            traverse_node_offset(node),
                            format!("'continue' cannot be used outside of a loop."),
                        ));
                    }

                    Op::Continue.into()
                }
                "next" => Op::Next.into(),
                "nop" => ImlOp::Nop,
                "reject" => Op::Reject.into(),

                "binary" => {
                    let children = node["children"].borrow();
                    let children = children.object::<List>().unwrap();
                    assert_eq!(children.len(), 2);

                    let (left, right) = (children[0].borrow(), children[1].borrow());

                    let left = traverse_node_rvalue(
                        compiler,
                        &left.object::<Dict>().unwrap(),
                        Rvalue::CallOrLoad,
                    );
                    let right = traverse_node_rvalue(
                        compiler,
                        &right.object::<Dict>().unwrap(),
                        Rvalue::CallOrLoad,
                    );

                    // When both results are values, calculate in-place
                    if let (Ok(left), Ok(right)) =
                        (left.get_evaluable_value(), right.get_evaluable_value())
                    {
                        return ImlOp::load(
                            traverse_node_offset(node),
                            ImlValue::from(left.binary_op(right, parts[2]).unwrap()),
                        );
                    }

                    // Otherwise, generate operational code
                    ops.push(left);
                    ops.push(right);

                    // Push operation position here
                    //ops.push(traverse_offset(node));

                    ImlOp::from(match parts[2] {
                        "add" => Op::BinaryOp("add"),
                        "sub" => Op::BinaryOp("sub"),
                        "mul" => Op::BinaryOp("mul"),
                        "div" => Op::BinaryOp("div"),
                        _ => {
                            unimplemented!("op_binary_{}", parts[2]);
                        }
                    })
                }

                "unary" => {
                    let children = node["children"].borrow();
                    let children = children.object::<Dict>().unwrap();

                    let res = traverse_node_rvalue(compiler, children, Rvalue::CallOrLoad);

                    if let Ok(value) = res.get_evaluable_value() {
                        return ImlOp::load(
                            traverse_node_offset(node),
                            ImlValue::from(value.unary_op(parts[2]).unwrap()),
                        );
                    }

                    // Push operation position here
                    //ops.push(traverse_offset(node));

                    ops.push(res);

                    ImlOp::from(match parts[2] {
                        "not" => Op::UnaryOp("not"),
                        "neg" => Op::UnaryOp("neg"),
                        _ => {
                            unimplemented!("op_unary_{}", parts[2]);
                        }
                    })
                }

                "compare" | "logical" => {
                    let children = node["children"].borrow();
                    let children = children.object::<List>().unwrap();
                    assert_eq!(children.len(), 2);

                    let (left, right) = (children[0].borrow(), children[1].borrow());
                    let left = traverse_node_rvalue(
                        compiler,
                        &left.object::<Dict>().unwrap(),
                        Rvalue::CallOrLoad,
                    );
                    let right = traverse_node_rvalue(
                        compiler,
                        &right.object::<Dict>().unwrap(),
                        Rvalue::CallOrLoad,
                    );

                    // When both results are values, compare in-place
                    if let (Ok(left), Ok(right)) =
                        (left.get_evaluable_value(), right.get_evaluable_value())
                    {
                        return ImlOp::load(
                            traverse_node_offset(node),
                            ImlValue::from(left.binary_op(right, parts[2]).unwrap()),
                        );
                    }

                    // Otherwise, generate operational code
                    ops.push(left);

                    match parts[2] {
                        "and" => ImlOp::If {
                            peek: true,
                            test: true,
                            then: Box::new(right),
                            else_: Box::new(ImlOp::from(Op::PushVoid)),
                        },
                        "or" => ImlOp::If {
                            peek: true,
                            test: false,
                            then: Box::new(right),
                            else_: Box::new(ImlOp::from(Op::PushVoid)),
                        },
                        _ => {
                            ops.push(right);
                            ImlOp::from(match parts[2] {
                                "eq" => Op::BinaryOp("eq"),
                                "neq" => Op::BinaryOp("neq"),
                                "lteq" => Op::BinaryOp("lteq"),
                                "gteq" => Op::BinaryOp("gteq"),
                                "lt" => Op::BinaryOp("lt"),
                                "gt" => Op::BinaryOp("gt"),
                                _ => {
                                    unimplemented!("op_compare_{}", parts[2]);
                                }
                            })
                        }
                    }
                }

                "mod" => {
                    let children = node["children"].borrow();
                    let children = children.object::<Dict>().unwrap();

                    let res = traverse_node_rvalue(compiler, children, Rvalue::CallOrLoad);

                    if !res.is_consuming() {
                        compiler.errors.push(Error::new(
                            traverse_node_offset(node),
                            format!(
                                "Operator '{}' has no effect on non-consuming {}",
                                match parts[2] {
                                    "pos" => "+",
                                    "kle" => "*",
                                    "opt" => "?",
                                    other => other,
                                },
                                if matches!(
                                    res,
                                    ImlOp::Load {
                                        target: ImlTarget::Static(_),
                                        ..
                                    } | ImlOp::Call {
                                        target: ImlTarget::Static(_),
                                        ..
                                    }
                                ) {
                                    "value"
                                } else {
                                    "sequence"
                                }
                            ),
                        ));
                    } else {
                        compiler.mark_consuming();
                    }

                    // Special operations for Token::Char
                    /*
                    if let ImlOp::Call(ImlOpValue(value), ..) = &res {
                        // In case of a token, try to replace it with a repeating counterpart.
                        if let ImlValue::Value(value) = value {
                            let value = value.borrow();

                            if let Some(token) = value.object::<Token>() {
                                if let Token::Char(ccl) = token.clone() {
                                    match parts[2] {
                                        // mod_pos on Token::Char becomes Token::Chars
                                        "pos" | "kle" => {
                                            let mut chars = ImlOp::Call(ImlOpValue(ImlValue::from(RefValue::from(Token::Chars(ccl)))), 0, false, traverse_node_offset(node));
                                            if parts[2] == "kle" {
                                                // mod_kle on Token::Char becomes Token::Chars.into_optional()
                                                chars = chars.into_optional();
                                            }

                                            return chars;
                                        }

                                        // mod_not on Token::Char becomes negated Token::Char
                                        "not" => {
                                            return ImlOp::Call(ImlOpValue(ImlValue::from(RefValue::from(Token::Char(ccl.negate())))), 0, false, traverse_node_offset(node));
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                    */

                    match parts[2] {
                        "pos" => res.into_positive(),
                        "kle" => res.into_kleene(),
                        "opt" => res.into_optional(),
                        "peek" => res.into_peek(),
                        "expect" => {
                            // Just give some helpful information here for most cases;
                            // `expect` will be replaced by the `Expect<P, msg>` generic parselet in future.
                            let msg = match &res {
                                ImlOp::Load {
                                    target: ImlTarget::Static(expect),
                                    ..
                                }
                                | ImlOp::Call {
                                    target: ImlTarget::Static(expect),
                                    ..
                                } => Some(format!("Expecting {:?}", expect)),
                                _ => None,
                            };

                            res.into_expect(msg)
                        }
                        "not" => res.into_not(),
                        _ => unreachable!(),
                    }
                }

                "if" => {
                    let children = node["children"].borrow();
                    let children = children.object::<List>().unwrap();

                    let (condition, then_part) = (children[0].borrow(), children[1].borrow());

                    let condition = traverse_node_rvalue(
                        compiler,
                        &condition.object::<Dict>().unwrap(),
                        Rvalue::CallOrLoad,
                    );
                    let then_part = traverse_node_rvalue(
                        compiler,
                        &then_part.object::<Dict>().unwrap(),
                        Rvalue::CallOrLoad,
                    );
                    let else_part = if children.len() == 3 {
                        let else_part = children[2].borrow();
                        traverse_node_rvalue(
                            compiler,
                            &else_part.object::<Dict>().unwrap(),
                            Rvalue::CallOrLoad,
                        )
                    } else {
                        ImlOp::from(Op::PushVoid)
                    };

                    // Compile time evaluation; When the if fails, it doesn't need
                    // to be compiled into the program.
                    if let Ok(value) = condition.get_evaluable_value() {
                        if value.is_true() {
                            return then_part;
                        }

                        return else_part;
                    }

                    ops.push(condition);

                    ImlOp::If {
                        peek: false,
                        test: true,
                        then: Box::new(then_part),
                        else_: Box::new(else_part),
                    }
                }

                "for" => {
                    let children = node["children"].borrow();
                    let children = children.object::<List>().unwrap();

                    let (initial, condition, each, body) = (
                        &children[0].borrow(),
                        &children[1].borrow(),
                        &children[2].borrow(),
                        &children[3].borrow(),
                    );

                    let initial = initial.object::<Dict>().unwrap();
                    let condition = condition.object::<Dict>().unwrap();
                    let each = each.object::<Dict>().unwrap();
                    let body = body.object::<Dict>().unwrap();

                    // Initial
                    let initial = traverse_node_rvalue(compiler, initial, Rvalue::CallOrLoad);

                    compiler.push_loop();

                    let condition = traverse_node_rvalue(compiler, condition, Rvalue::CallOrLoad);
                    let body = ImlOp::from(vec![
                        traverse_node_rvalue(compiler, body, Rvalue::Load),
                        traverse_node_rvalue(compiler, each, Rvalue::CallOrLoad),
                    ]);

                    compiler.pop_loop();

                    ImlOp::Loop {
                        consuming: None,
                        init: Box::new(initial),
                        condition: Box::new(condition),
                        body: Box::new(body),
                    }
                }

                "loop" => {
                    let children = List::from(&node["children"]);

                    compiler.push_loop();

                    let ret = match children.len() {
                        1 => {
                            let body = &children[0].borrow();

                            ImlOp::Loop {
                                consuming: None,
                                init: Box::new(ImlOp::Nop),
                                condition: Box::new(ImlOp::Nop),
                                body: Box::new(traverse_node_rvalue(
                                    compiler,
                                    body.object::<Dict>().unwrap(),
                                    Rvalue::CallOrLoad,
                                )),
                            }
                        }
                        2 => {
                            let (condition, body) = (&children[0].borrow(), &children[1].borrow());

                            ImlOp::Loop {
                                consuming: None,
                                init: Box::new(ImlOp::Nop),
                                condition: Box::new(traverse_node_rvalue(
                                    compiler,
                                    condition.object::<Dict>().unwrap(),
                                    Rvalue::CallOrLoad,
                                )),
                                body: Box::new(traverse_node_rvalue(
                                    compiler,
                                    body.object::<Dict>().unwrap(),
                                    Rvalue::CallOrLoad,
                                )),
                            }
                        }
                        _ => unreachable!(),
                    };

                    compiler.pop_loop();

                    ret
                }

                _ => {
                    unimplemented!("{} missing", op);
                }
            };
            ops.push(op);

            ImlOp::from(ops)
        }

        // sequence  ------------------------------------------------------
        "sequence" | "list" => {
            let children = if let Some(children) = node.get("children") {
                List::from(children)
            } else {
                List::new()
            };

            let mut ops = Vec::new();

            for node in children.iter() {
                ops.push(traverse_node_rvalue(
                    compiler,
                    node.borrow().object::<Dict>().unwrap(),
                    Rvalue::CallOrLoad,
                ))
            }

            if emit == "sequence" {
                if ops.len() == 1 {
                    ImlOp::from(ops)
                } else if ops.len() > 0 {
                    ImlOp::Seq {
                        seq: ops,
                        framed: true,
                    }
                } else {
                    ImlOp::Nop
                }
            } else {
                ops.push(Op::MakeList(children.len()).into());
                ImlOp::from(ops)
            }
        }

        // ---------------------------------------------------------------
        _ => {
            // When there are children, try to traverse_node_or_list recursively
            if let Some(children) = node.get("children") {
                traverse_node_or_list(compiler, children)
            }
            // Otherwise, report unhandled node!
            else {
                unreachable!("No handling for {:?}", node);
            }
        }
    }
}

/// Debug function to print an AST to stdout.
pub fn print(ast: &RefValue) {
    fn print(value: &RefValue, indent: usize) {
        let value = value.borrow();

        if let Some(d) = value.object::<Dict>() {
            let emit = d["emit"].to_string();

            let row = d
                .get("row")
                .and_then(|row| Some(row.borrow().to_usize().unwrap()));
            let col = d
                .get("col")
                .and_then(|col| Some(col.borrow().to_usize().unwrap()));
            let stop_row = d
                .get("stop_row")
                .and_then(|row| Some(row.borrow().to_usize().unwrap()));
            let stop_col = d
                .get("stop_col")
                .and_then(|col| Some(col.borrow().to_usize().unwrap()));

            let value = d.get("value");
            let children = d.get("children");

            if let (Some(row), Some(col), Some(stop_row), Some(stop_col)) =
                (row, col, stop_row, stop_col)
            {
                print!(
                    "{:indent$}{} [start {}:{}, end {}:{}]",
                    "",
                    emit,
                    row,
                    col,
                    stop_row,
                    stop_col,
                    indent = indent
                );
            } else if let (Some(row), Some(col)) = (row, col) {
                print!("{:indent$}{} [{}:{}]", "", emit, row, col, indent = indent);
            } else {
                print!("{:indent$}{}", "", emit, indent = indent);
            }

            if let Some(value) = value {
                print!(" => {}", value.repr());
            }
            print!("\n");

            if let Some(children) = children {
                print(children, indent + 1);
            }
        } else if let Some(l) = value.object::<List>() {
            for item in l.iter() {
                print(item, indent);
            }
        }
    }

    print(ast, 0);
}

tokay_function!("ast(emit, value=void, flatten=true)", {
    let context = context.unwrap();

    let mut ret = Dict::new();
    ret.insert("emit".to_string(), emit);

    let value = if value.is_void() {
        context
            .collect(context.capture_start, false, true, false, 0)
            .unwrap_or(None)
    } else {
        Some(value)
    };

    if let Some(value) = value {
        // Lists can be flattened
        if value.borrow().object::<List>().is_some() {
            ret.insert(
                "children".to_string(),
                if flatten.is_true() {
                    List::list_flatten(vec![value], None).unwrap()
                } else {
                    value.clone()
                },
            );
        }
        // Dicts can be used directly as children
        else if value.borrow().object::<Dict>().is_some() {
            ret.insert("children".to_string(), value.clone());
        }
        // Otherwise this is a value
        else {
            ret.insert("value".to_string(), value.clone());
        }
    }

    // Store positions of reader start
    ret.insert(
        "offset".to_string(),
        value!(context.reader_start.offset + context.runtime.start),
    );
    ret.insert("row".to_string(), value!(context.reader_start.row as usize));
    ret.insert("col".to_string(), value!(context.reader_start.col as usize));

    // Store positions of reader stop
    let current = context.runtime.reader.tell();

    ret.insert(
        "stop_offset".to_string(),
        value!(current.offset + context.runtime.start),
    );
    ret.insert("stop_row".to_string(), value!(current.row as usize));
    ret.insert("stop_col".to_string(), value!(current.col as usize));

    RefValue::from(ret).into()
});

tokay_function!("ast_print(ast)", {
    print(&ast);
    value!(void).into()
});
