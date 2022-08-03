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

        ImlOp::from_vec(ops)
    } else if let Some(dict) = ast.borrow().object::<Dict>() {
        traverse_node(compiler, dict)
    } else {
        ImlOp::Load(ImlOpValue(ImlValue::from(RefValue::from(ast.clone()))))
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

    // ... because in case ImlResult::Ops is returned here, it would be nice to have it in a separate scope.
    match traverse_node(compiler, node) {
        ImlOp::Nop => {
            compiler.pop_parselet(None, None, None, None, None, ImlOp::from(Op::Nop));
            value!(void).into()
        }
        // Defined parselet or value
        ImlOp::Load(ImlOpValue(value)) => {
            compiler.pop_parselet(None, None, None, None, None, ImlOp::from(Op::Nop));

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

    ImlOp::from_vec(ops)
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

            let (left, right) = (children[0].borrow(), children[1].borrow());

            let left = traverse_node(compiler, &left.object::<Dict>().unwrap());
            let right = traverse_node(compiler, &right.object::<Dict>().unwrap());

            // Push value first, then the alias
            ImlOp::from_vec(vec![right, left, ImlOp::from(Op::MakeAlias)])
        }

        // assign ---------------------------------------------------------
        assign if assign.starts_with("assign") => {
            let children = node["children"].borrow();
            let children = children.object::<List>().unwrap();

            let (lvalue, value) = (children[0].borrow(), children[1].borrow());
            let lvalue = lvalue.object::<Dict>().unwrap();
            let value = value.object::<Dict>().unwrap();

            let parts: Vec<&str> = assign.split("_").collect();

            let mut ops = vec![traverse_offset(node)];

            if parts.len() > 1 && parts[1] != "hold" {
                ops.push(traverse_node_lvalue(compiler, lvalue, false, false));
                ops.push(traverse_node(compiler, value));

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
                ops.push(traverse_node(compiler, value));
                ops.push(traverse_node_lvalue(
                    compiler,
                    lvalue,
                    true,
                    *parts.last().unwrap() == "hold",
                ));
            }

            ImlOp::from_vec(ops)
        }

        // attribute ------------------------------------------------------
        "attribute" => ImlOp::from_vec(vec![
            traverse_node_or_list(compiler, &node["children"]),
            traverse_offset(node),
            ImlOp::from(Op::LoadAttr),
        ]),
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

                    ImlOp::Call(ImlOpValue(main), 0, false)
                }
            } else {
                ImlOp::Nop
            }
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

                        ops.push(traverse_node_or_list(compiler, &param["children"]));
                        args += 1;
                    }

                    "param_named" => {
                        let children = List::from(&param["children"]);

                        ops.push(traverse_node_or_list(compiler, &children[1]));

                        let ident = children[0].borrow();
                        let ident = ident.object::<Dict>().unwrap();
                        let ident = ident["value"].borrow();
                        let ident = ident.object::<Str>().unwrap().as_str();

                        ops.push(ImlOp::Load(ImlOpValue(ImlValue::from(RefValue::from(
                            ident,
                        )))));

                        nargs += 1;
                    }

                    other => unimplemented!("Unhandled parameter type {:?}", other),
                }
            }

            // When calling with nargs, create a nargs dict first
            if nargs > 0 {
                ops.push(ImlOp::from(Op::MakeDict(nargs)));
            }

            // Perform static call or resolved rvalue call
            let callee = traverse_node_or_list(compiler, &children[0]);

            todo!();

            /*
            if let ImlResult::Identifier(ident, offset) = callee {
                if utils::identifier_is_consumable(&ident) {
                    compiler.mark_consuming();
                }

                ops.push(
                    Usage::Call {
                        name: ident.to_string(),
                        args,
                        nargs,
                        offset,
                    }
                    .resolve_or_dispose(compiler),
                )
            } else {
                ops.push(callee);

                if args == 0 && nargs == 0 {
                    ops.push(Op::Call.into());
                } else if args > 0 && nargs == 0 {
                    ops.push(Op::CallArg(args).into());
                } else {
                    ops.push(Op::CallArgNamed(args).into())
                }
            }
            */

            ImlOp::from_vec(ops)
        }

        // capture --------------------------------------------------------
        "capture_alias" | "capture_expr" => ImlOp::from_vec(vec![
            traverse_node_or_list(compiler, &node["children"]),
            ImlOp::from(Op::LoadCapture),
        ]),

        "capture_index" => {
            let children = node["children"].borrow();
            let children = children.object::<Dict>().unwrap();
            let index = traverse_node_value(compiler, children).value();
            ImlOp::from(Op::LoadFastCapture(index.to_usize().unwrap()))
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

        // identifier -----------------------------------------------------
        "identifier" => {
            let name = node["value"].borrow();
            let name = name.object::<Str>().unwrap().as_str();

            // Check if identifier is valid
            if let Err(mut error) = identifier_is_valid(name) {
                if let Some(offset) = traverse_node_offset(node) {
                    error.patch_offset(offset);
                }

                compiler.errors.push(error);
                ImlOp::Nop
            } else {
                let offset = traverse_node_offset(node);

                let usage = if false
                /* todo: was previously if call from ImlResult */
                {
                    Usage::CallOrCopy {
                        name: name.to_string(),
                        offset,
                    }
                } else {
                    Usage::Load {
                        name: name.to_string(),
                        offset,
                    }
                };

                usage.resolve_or_dispose(compiler)
            }
        }

        // index ----------------------------------------------------------
        "index" => {
            ImlOp::from_vec(vec![
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

            ImlOp::from_vec(ops)
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
                        ops.push(traverse_node(compiler, &value.object::<Dict>().unwrap()));

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

                    let left = traverse_node(compiler, &left.object::<Dict>().unwrap());
                    let right = traverse_node(compiler, &right.object::<Dict>().unwrap());

                    // When both results are values, calculate in-place
                    if let (Ok(left), Ok(right)) =
                        (left.get_evaluable_value(), right.get_evaluable_value())
                    {
                        return ImlOp::Load(ImlOpValue(ImlValue::from(
                            left.binary_op(right, parts[2]).unwrap(),
                        )));
                    }

                    // Push operation position here
                    ops.push(traverse_offset(node));

                    // Otherwise, generate operational code
                    ops.push(left);
                    ops.push(right);

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

                    let res = traverse_node(compiler, children);

                    if let Ok(value) = res.get_evaluable_value() {
                        return ImlOp::Load(ImlOpValue(ImlValue::from(
                            value.unary_op(parts[2]).unwrap(),
                        )));
                    }

                    // Push operation position here
                    ops.push(traverse_offset(node));
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
                    let left = traverse_node(compiler, &left.object::<Dict>().unwrap());
                    let right = traverse_node(compiler, &right.object::<Dict>().unwrap());

                    // When both results are values, compare in-place
                    if let (Ok(left), Ok(right)) =
                        (left.get_evaluable_value(), right.get_evaluable_value())
                    {
                        return ImlOp::Load(ImlOpValue(ImlValue::from(
                            left.binary_op(right, parts[2]).unwrap(),
                        )));
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

                    let res = traverse_node(compiler, children);

                    todo!();

                    /*
                    if !res.is_consuming() {
                        compiler.errors.push(Error::new(
                            traverse_node_offset(node),
                            format!(
                                "Operator '{}' has no effect on non-consuming value",
                                parts[2]
                            ),
                        ));
                    } else {
                        compiler.mark_consuming();
                    }

                    // Special operations for Token::Char
                    if let ImlOp::Call(value) = &res {
                        // In case of a token, try to replace it with a repeating counterpart.
                        if let ImlValue::Value(value) = value {
                            let value = value.borrow();

                            if let Some(token) = value.object::<Token>() {
                                if let Token::Char(ccl) = token.clone() {
                                    match parts[2] {
                                        // mod_pos on Token::Char becomes Token::Chars
                                        "pos" | "kle" => {
                                            let chars = ImlValue::Call(
                                                RefValue::from(Token::Chars(ccl)).into(),
                                            );

                                            let mut ops =
                                                ImlOp::from_vec(chars.into_ops(compiler, true));

                                            if parts[2] == "kle" {
                                                // mod_kle on Token::Char becomes Token::Chars.into_optional()
                                                ops = ops.into_optional();
                                            }

                                            return ImlResult::Ops(vec![ops]);
                                        }

                                        // mod_not on Token::Char becomes negated Token::Char
                                        "not" => {
                                            return ImlResult::Value(
                                                RefValue::from(Token::Char(ccl.negate())).into(),
                                            );
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }

                    // This reports error when modifier is used with non-consuming code
                    // (for now disabled and under further consideration)
                    /*
                    if !scope.consuming {
                        compiler.errors.push(Error::new(
                            traverse_node_offset(node),
                            format!(
                                "Operator '{}' has no effect on non-consuming {}",
                                match parts[2] {
                                    "pos" => "+",
                                    "kle" => "*",
                                    "opt" => "?",
                                    other => other
                                },
                                if matches!(res, ImlResult::Value(_)) {
                                    "value"
                                }
                                else {
                                    "code"
                                }
                            ),
                        ));
                    }
                    else {
                        compiler.scopes[0].consuming = true;
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
                            let msg = if let ImlOp::Op(Op::CallStatic(i)) = &op {
                                Some(format!("Expecting {}", compiler.values[*i]))
                            } else {
                                None
                            };

                            op.into_expect(msg)
                        }
                        "not" => op.into_not(),
                        _ => unreachable!(),
                    }
                    */
                }

                "if" => {
                    let children = node["children"].borrow();
                    let children = children.object::<List>().unwrap();

                    let condition = traverse_node_or_list(compiler, &children[0]);
                    let then = traverse_node_or_list(compiler, &children[1]);
                    let else_ = if children.len() == 3 {
                        Some(traverse_node_or_list(compiler, &children[2]))
                    } else {
                        None
                    };

                    // Compile time evaluation; When the if fails, it doesn't need
                    // to be compiled into the program.
                    if let Ok(value) = condition.get_evaluable_value() {
                        if value.is_true() {
                            return then;
                        } else if let Some(else_) = else_ {
                            return else_;
                        }

                        return ImlOp::Load(ImlOpValue(ImlValue::from(value!(void))));
                    }

                    ops.push(condition);

                    ImlOp::If {
                        peek: false,
                        test: true,
                        then: Box::new(then),
                        else_: Box::new(else_.unwrap_or(ImlOp::from(Op::PushVoid))),
                    }
                }

                "for" => {
                    let children = node["children"].borrow();
                    let children = children.object::<List>().unwrap();

                    // Initial
                    let initial = traverse_node_or_list(compiler, &children[0]);

                    compiler.push_loop();

                    let condition = traverse_node_or_list(compiler, &children[1]);
                    let body = ImlOp::from_vec(vec![
                        traverse_node_or_list(compiler, &children[3]),
                        traverse_node_or_list(compiler, &children[2]),
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
                        1 => ImlOp::Loop {
                            consuming: None,
                            init: Box::new(ImlOp::Nop),
                            condition: Box::new(ImlOp::Nop),
                            body: Box::new(traverse_node_or_list(compiler, &children[0])),
                        },
                        2 => ImlOp::Loop {
                            consuming: None,
                            init: Box::new(ImlOp::Nop),
                            condition: Box::new(traverse_node_or_list(compiler, &children[0])),
                            body: Box::new(traverse_node_or_list(compiler, &children[1])),
                        },
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

            ImlOp::from_vec(ops)
        }

        // rvalue ---------------------------------------------------------
        "rvalue" => {
            let children = List::from(&node["children"]);

            let mut ops = Vec::new();

            for node in children.iter() {
                ops.push(traverse_node_or_list(compiler, node));
            }

            assert!(ops.len() > 0);
            ImlOp::from_vec(ops)
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
                ops.push(traverse_node_or_list(compiler, node))
            }

            if emit == "sequence" {
                if ops.len() == 1 {
                    ImlOp::from_vec(ops)
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
                ImlOp::from_vec(ops)
            }
        }

        // value ---------------------------------------------------------
        value if value.starts_with("value_") => {
            ImlOp::Load(ImlOpValue(traverse_node_value(compiler, node)))
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
