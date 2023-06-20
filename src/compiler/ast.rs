//! Compiler's internal Abstract Syntax Tree traversal
use indexmap::IndexMap;
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
pub fn identifier_is_valid(ident: &str) -> Result<(), Error> {
    match ident {
        "Char" | "Chars" | "accept" | "begin" | "break" | "continue" | "else" | "end" | "exit"
        | "expect" | "false" | "for" | "if" | "in" | "loop" | "next" | "not" | "null" | "peek"
        | "push" | "reject" | "repeat" | "reset" | "return" | "self" | "Self" | "true" | "void" => {
            Err(Error::new(
                None,
                format!("Expected identifier, found reserved word '{}'", ident),
            ))
        }
        _ => Ok(()),
    }
}

/// AST traversal entry
pub(in crate::compiler) fn traverse(compiler: &mut Compiler, ast: &RefValue) -> ImlOp {
    if let Some(list) = ast.borrow().object::<List>() {
        let mut ops = Vec::new();

        for item in list.iter() {
            ops.push(traverse(compiler, item));
        }

        ImlOp::from(ops)
    } else if let Some(dict) = ast.borrow().object::<Dict>() {
        traverse_node_rvalue(compiler, dict, Rvalue::CallOrLoad)
    } else {
        ImlOp::load(compiler, None, ImlValue::from(RefValue::from(ast.clone())))
    }
}

// Extract offset positions into an Offset structure
fn traverse_node_offset(node: &Dict) -> Option<Offset> {
    //return None; // Temporarily discard any Offset information (shortens debug output)

    let offset = node
        .get_str("offset")
        .and_then(|offset| Some(offset.to_usize().unwrap()));
    let row = node
        .get_str("row")
        .and_then(|row| Some(row.to_usize().unwrap() as u32));
    let col = node
        .get_str("col")
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
        "value_void" => ImlValue::Value(compiler.statics[0].clone()),
        "value_null" => ImlValue::Value(compiler.statics[1].clone()),
        "value_true" => ImlValue::Value(compiler.statics[2].clone()),
        "value_false" => ImlValue::Value(compiler.statics[3].clone()),
        "value_self" => ImlValue::This(false),
        "value_integer" => match node["value"].to_i64() {
            Ok(0) => ImlValue::Value(compiler.statics[4].clone()),
            Ok(1) => ImlValue::Value(compiler.statics[5].clone()),
            _ => compiler.register_static(node["value"].clone()),
        },
        "value_float" => compiler.register_static(node["value"].clone()),
        "value_string" => compiler.register_static(node["value"].clone()),

        // Tokens
        "value_token_self" => ImlValue::This(true),
        "value_token_match" | "value_token_touch" => {
            let mut value = node["value"].to_string();

            if value.len() == 0 {
                compiler.errors.push(Error::new(
                    traverse_node_offset(node),
                    format!("Empty match not allowed"),
                ));
                value = "#INVALID".to_string();
            }

            compiler.register_static(if emit == "value_token_match" {
                RefValue::from(Token::Match(value))
            } else {
                RefValue::from(Token::Touch(value))
            })
        }
        "value_token_any" => {
            compiler.register_static(RefValue::from(Token::Char(CharClass::new().negate())))
        }
        "value_token_anys" => {
            compiler.register_static(RefValue::from(Token::Chars(CharClass::new().negate())))
        }
        "value_token_ccl" | "value_token_ccls" => {
            let many = emit.ends_with("s");

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
                ccl = ccl.negate();
            } else {
                assert!(emit == "ccl");
            }

            compiler.register_static(if many {
                RefValue::from(Token::Chars(ccl))
            } else {
                RefValue::from(Token::Char(ccl))
            })
        }

        // Parselets
        "value_parselet" => {
            compiler.parselet_push();

            let mut constants: IndexMap<String, ImlValue> = IndexMap::new();
            let mut signature: IndexMap<String, ImlValue> = IndexMap::new();

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
                        let offset = traverse_node_offset(node);

                        // check if identifier was not provided twice
                        if constants.contains_key(&ident) {
                            compiler.errors.push(Error::new(
                                offset,
                                format!("Generic '{}' already given in signature before", ident),
                            ));

                            continue;
                        }

                        compiler.set_constant(
                            &ident,
                            ImlValue::Name {
                                offset,
                                generic: true,
                                name: ident.to_string(),
                            },
                        );

                        assert!(children.len() <= 2);

                        constants.insert(
                            ident.to_string(),
                            if children.len() == 2 {
                                let default = children[1].borrow();
                                traverse_node_static(
                                    compiler,
                                    Some(&ident),
                                    default.object::<Dict>().unwrap(),
                                )
                            } else {
                                ImlValue::Void
                            },
                        );
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
                        if signature.contains_key(&ident) {
                            compiler.errors.push(Error::new(
                                traverse_node_offset(node),
                                format!("Argument '{}' already given in signature before", ident),
                            ));

                            continue;
                        }

                        compiler.new_local(&ident);

                        assert!(children.len() <= 2);

                        signature.insert(
                            ident.to_string(),
                            if children.len() == 2 {
                                let default = children[1].borrow();
                                traverse_node_static(
                                    compiler,
                                    Some(&ident),
                                    default.object::<Dict>().unwrap(),
                                )
                            } else {
                                ImlValue::Void
                            },
                        );
                        //println!("{} {} {:?}", emit.to_string(), ident, default);
                    }
                    _ => unreachable!(),
                }
            }

            let body = body.borrow();
            let body =
                traverse_node_rvalue(compiler, body.object::<Dict>().unwrap(), Rvalue::CallOrLoad);

            let ret = compiler.parselet_pop(
                traverse_node_offset(node),
                None,
                None,
                Some(constants),
                Some(signature),
                body,
            );

            //println!("parselet = {:#?}", ret);
            ret
        }

        "value_generic" => {
            let children = List::from(&node["children"]);

            // Traverse the target
            let target = &children[0].borrow();
            let target = target.object::<Dict>().unwrap();
            let target = traverse_node_static(compiler, None, target);

            // Traverse generic arguments
            let mut args = Vec::new();
            let mut nargs = IndexMap::new();

            for genarg in children[1..].iter() {
                let genarg = genarg.borrow();
                let genarg = genarg.object::<Dict>().unwrap();

                let offset = traverse_node_offset(genarg);
                let emit = genarg["emit"].borrow();

                match emit.object::<Str>().unwrap().as_str() {
                    "genarg" => {
                        if !nargs.is_empty() {
                            compiler.errors.push(Error::new(
                                traverse_node_offset(node),
                                format!(
                                    "Sequencial generics need to be specified before named generics."
                                ),
                            ));

                            continue;
                        }

                        let param = &genarg["children"].borrow();
                        let param = param.object::<Dict>().unwrap();

                        args.push((offset, traverse_node_static(compiler, None, param)));
                    }

                    "genarg_named" => {
                        let children = List::from(&genarg["children"]);

                        let ident = children[0].borrow();
                        let ident = ident.object::<Dict>().unwrap();
                        let ident = ident["value"].borrow();
                        let ident = ident.object::<Str>().unwrap().as_str();

                        if nargs.contains_key(ident) {
                            compiler.errors.push(Error::new(
                                traverse_node_offset(genarg),
                                format!("Named generic '{}' provided more than once.", ident),
                            ));

                            continue;
                        }

                        let param = &children[1].borrow();
                        let param = param.object::<Dict>().unwrap();

                        nargs.insert(
                            ident.to_string(),
                            (offset, traverse_node_static(compiler, None, param)),
                        );
                    }

                    other => unimplemented!("Unhandled genarg type {:?}", other),
                }
            }

            let mut ret = ImlValue::Instance {
                target: Box::new(target),
                args,
                nargs,
                offset: traverse_node_offset(node),
            };

            ret.resolve(compiler);
            ret
        }

        _ => unimplemented!("unhandled value node {}", emit),
    }
}

/** Traverse a static value.
The value must either be a literal or something from a known constant.

The name attribute is optional and can be used to assign an identifier to parselets for debug purposes
*/
fn traverse_node_static(compiler: &mut Compiler, lvalue: Option<&str>, node: &Dict) -> ImlValue {
    compiler.parselet_push(); // yep, we push a parselet scope here...

    match traverse_node_rvalue(compiler, node, Rvalue::Load) {
        ImlOp::Nop => {
            compiler.parselet_pop(None, None, None, None, None, ImlOp::Nop);
            value!(void).into()
        }
        // Defined parselet or value
        ImlOp::Load { target: value, .. } => {
            compiler.parselet_pop(None, None, None, None, None, ImlOp::Nop);

            if let Some(lvalue) = lvalue {
                if let ImlValue::Parselet(parselet) = &value {
                    let mut parselet = parselet.borrow_mut();
                    parselet.name = Some(lvalue.to_string());
                }
            }

            value
        }

        // Any other code becomes its own parselet without any signature.
        other => compiler.parselet_pop(
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

        let last = i == children.len() - 1; // Identify last child
        let store = if !last { false } else { store }; // Identify store instruction, otherwise load

        match emit {
            capture if capture.starts_with("capture") => {
                let children = item["children"].borrow();

                match capture {
                    "capture_alias" | "capture_expr" => {
                        ops.push(traverse_node_rvalue(
                            compiler,
                            children.object::<Dict>().unwrap(),
                            Rvalue::CallOrLoad,
                        ));

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
                        let index = traverse_node_value(compiler, children).unwrap();

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

                /* Generates code for a symbol store, which means:

                    1. look-up local variable, and store into
                    2. look-up global variable, and store into
                    3. create local variable, and store into
                */
                match compiler.get(name) {
                    // Known local
                    Some(ImlValue::Local(addr)) => {
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
                    // Known global
                    Some(ImlValue::Global(addr)) => {
                        if store {
                            if hold {
                                ops.push(Op::StoreGlobalHold(addr).into())
                            } else {
                                ops.push(Op::StoreGlobal(addr).into())
                            }
                        } else {
                            ops.push(Op::LoadGlobal(addr).into())
                        }
                    }
                    // Check for not assigning to a constant (at any level)
                    Some(_) => {
                        compiler.errors.push(Error::new(
                            traverse_node_offset(node),
                            format!("Cannot assign to constant '{}'", name),
                        ));
                        break;
                    }
                    // Undefined name
                    None => {
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

                        // When chained lvalue, name must be declared!
                        if children.len() > 1 {
                            compiler.errors.push(Error::new(
                                traverse_node_offset(node),
                                format!("Undeclared variable '{}', please define it first", name),
                            ));
                            break;
                        }

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
            }

            // item -----------------------------------------------------------
            "item" => {
                ops.push(traverse(compiler, &item["children"]));

                if store {
                    if hold {
                        ops.push(Op::StoreItemHold.into());
                    } else {
                        ops.push(Op::StoreItem.into());
                    }
                } else {
                    ops.push(Op::LoadItem.into())
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

    //println!("emit = {:?}", emit);

    let op = match emit {
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
                        ImlOp::call_by_name(compiler, offset, name.to_string(), None)
                    }
                    Rvalue::Call(args, nargs) => {
                        ImlOp::call_by_name(compiler, offset, name.to_string(), Some((args, nargs)))
                    }
                }
            };
        }

        // item -----------------------------------------------------------
        "item" => ImlOp::from(vec![
            traverse(compiler, &node["children"]),
            traverse_offset(node),
            ImlOp::from(Op::LoadItem),
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

        // value ---------------------------------------------------------
        value if value.starts_with("value_") => {
            let offset = traverse_node_offset(node);
            let value = traverse_node_value(compiler, node);

            return match mode {
                Rvalue::Load => ImlOp::load(compiler, offset, value),
                Rvalue::CallOrLoad => ImlOp::call(compiler, offset, value, None),
                Rvalue::Call(args, nargs) => {
                    ImlOp::call(compiler, offset, value, Some((args, nargs)))
                }
            };
        }

        _ => return traverse_node(compiler, node),
    };

    // Rvalue call confguration of a value known at runtime.
    match mode {
        Rvalue::Load => op,
        Rvalue::CallOrLoad => ImlOp::from(vec![op, ImlOp::from(Op::CallOrCopy)]),
        Rvalue::Call(args, nargs) => ImlOp::from(vec![
            op,
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

fn traverse_node(compiler: &mut Compiler, node: &Dict) -> ImlOp {
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

        // area -----------------------------------------------------------
        "area" => {
            let body = traverse(compiler, &node["children"]);
            ImlOp::seq(vec![body, ImlOp::from(Op::Extend)], false)
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
                    "divi" => ImlOp::from(Op::BinaryOp("idivi")),
                    "mod" => ImlOp::from(Op::BinaryOp("imod")),
                    _ => unreachable!(),
                });

                if *parts.last().unwrap() != "hold" {
                    ops.push(Op::Inv.into());
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
            let body = traverse(compiler, &node["children"]);

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
        "block" | "body" | "main" => {
            if let Some(ast) = node.get_str("children") {
                if emit == "block" {
                    compiler.block_push();
                }
                // When interactive and there's a scope, don't push, as the main scope
                // is kept to hold globals.
                else if emit == "main" && compiler.scopes.len() != 1 {
                    compiler.parselet_push(); // Main
                }

                let body = if let Some(list) = ast.borrow().object::<List>() {
                    let mut alts = Vec::new();

                    for item in list.iter() {
                        match traverse(compiler, item) {
                            ImlOp::Nop => {}
                            item => alts.push(item),
                        }
                    }

                    ImlOp::Alt { alts }
                } else if let Some(dict) = ast.borrow().object::<Dict>() {
                    traverse_node_rvalue(compiler, dict, Rvalue::CallOrLoad)
                } else {
                    unreachable!();
                };

                match emit {
                    "block" => {
                        compiler.block_pop();
                        body
                    }
                    "main" => {
                        let main = compiler.parselet_pop(
                            None,
                            Some("__main__".to_string()),
                            None,
                            None,
                            None,
                            body,
                        );

                        ImlOp::call(compiler, None, main, None)
                    }
                    _ => body,
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
                    "callarg" => {
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

                    "callarg_named" => {
                        let children = List::from(&param["children"]);

                        let param = &children[1].borrow();
                        let param = param.object::<Dict>().unwrap();

                        ops.push(traverse_node_rvalue(compiler, param, Rvalue::CallOrLoad));

                        let ident = children[0].borrow();
                        let ident = ident.object::<Dict>().unwrap();
                        let ident = ident["value"].borrow();
                        let ident = ident.object::<Str>().unwrap().as_str();

                        ops.push(ImlOp::load(
                            compiler,
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
            let index = traverse_node_value(compiler, children.object::<Dict>().unwrap()).unwrap();
            ImlOp::from(Op::LoadFastCapture(index.to_usize().unwrap()))
        }

        // comparison -----------------------------------------------------
        "comparison" => {
            // comparison can be a chain of comparisons, allowing to compare e.g. `1 < 2 < 3`
            let children = node["children"].borrow();
            let mut children = children.object::<List>().unwrap().clone();

            let first = children.remove(0);
            let first = first.borrow();

            let mut ops = Vec::new();

            ops.push(traverse_node_rvalue(
                compiler,
                &first.object::<Dict>().unwrap(),
                Rvalue::CallOrLoad,
            ));

            let mut backpatch = Vec::new();

            while !children.is_empty() {
                let child = children.remove(0);
                let child = child.borrow();
                let child = child.object::<Dict>().unwrap();

                let emit = child["emit"].borrow();
                let emit = emit.object::<Str>().unwrap().as_str();

                let next = child["children"].borrow();

                ops.push(traverse_node_rvalue(
                    compiler,
                    &next.object::<Dict>().unwrap(),
                    Rvalue::CallOrLoad,
                ));

                // Chained comparison requires forperand duplication
                if !children.is_empty() {
                    ops.push(ImlOp::from(Op::Swap(2))); // Swap operands
                    ops.push(ImlOp::from(Op::Copy(2))); // Copy second operand
                }

                ops.push(ImlOp::from(match emit {
                    "cmp_eq" => Op::BinaryOp("eq"),
                    "cmp_neq" => Op::BinaryOp("neq"),
                    "cmp_lteq" => Op::BinaryOp("lteq"),
                    "cmp_gteq" => Op::BinaryOp("gteq"),
                    "cmp_lt" => Op::BinaryOp("lt"),
                    "cmp_gt" => Op::BinaryOp("gt"),
                    _ => unimplemented!("{}", emit),
                }));

                // Push and remember placeholder for later clean-up jump
                if !children.is_empty() {
                    backpatch.push(ops.len());
                    ops.push(ImlOp::Nop); // Placeholder for condition
                }
            }

            if backpatch.len() > 0 {
                // Jump over clean-up part with last result
                ops.push(ImlOp::from(Op::Forward(3)));

                // Otherwise, remember clean-up start
                let clean_up = ops.len();
                ops.push(ImlOp::from(Op::Drop));
                ops.push(ImlOp::from(Op::PushFalse));

                // Backpatch all placeholders to relative jump to the clean-up part
                for index in backpatch {
                    ops[index] = ImlOp::from(Op::ForwardIfFalse(clean_up - index + 1));
                }
            }

            ImlOp::from(ops)
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
                    ops.push(ImlOp::from(if parts[2] == "inc" {
                        Op::UnaryOp("iinc")
                    } else {
                        Op::UnaryOp("idec")
                    }));
                    ops.push(ImlOp::from(Op::Sep)); // Separate TOS
                }
                "post" => {
                    ops.extend(vec![
                        ImlOp::from(Op::Dup),
                        ImlOp::from(Op::Swap(2)),
                        ImlOp::from(if parts[2] == "inc" {
                            Op::UnaryOp("iinc")
                        } else {
                            Op::UnaryOp("idec")
                        }),
                        ImlOp::from(Op::Drop),
                    ]);
                }
                _ => unreachable!(),
            }

            ImlOp::from(ops)
        }

        // operator ------------------------------------------------------
        op if op.starts_with("op_") => {
            let parts: Vec<&str> = emit.split("_").collect();
            let mut ops = Vec::new();

            let op = match parts[1] {
                "accept" | "break" | "exit" | "push" => {
                    if parts[1] == "break" && !compiler.loop_check() {
                        compiler.errors.push(Error::new(
                            traverse_node_offset(node),
                            format!("'break' cannot be used outside of a loop."),
                        ));
                    }

                    if let Some(value) = node.get_str("children") {
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
                            _ => unreachable!(),
                        }
                    } else {
                        match parts[1] {
                            "accept" => Op::Accept.into(),
                            "break" => Op::Break.into(),
                            "exit" => Op::Exit.into(),
                            "push" => Op::Push.into(),
                            _ => unreachable!(),
                        }
                    }
                }

                "continue" => {
                    if !compiler.loop_check() {
                        compiler.errors.push(Error::new(
                            traverse_node_offset(node),
                            format!("'continue' cannot be used outside of a loop."),
                        ));
                    }

                    Op::Continue.into()
                }

                "deref" => {
                    let children = node["children"].borrow();
                    let children = children.object::<Dict>().unwrap();

                    traverse_node_rvalue(compiler, children, Rvalue::Load)
                }

                "next" => Op::Next.into(),

                "nop" => ImlOp::Nop,

                "reject" => Op::Reject.into(),

                "repeat" => Op::Repeat.into(),

                "reset" => Op::Reset.into(),

                "unary" => {
                    let children = node["children"].borrow();
                    let children = children.object::<Dict>().unwrap();

                    let res = traverse_node_rvalue(compiler, children, Rvalue::CallOrLoad);

                    // Evaluate operation at compile-time if possible
                    if let Ok(value) = res.get_evaluable_value() {
                        if let Ok(value) = value.unary_op(parts[2]) {
                            return ImlOp::load(
                                compiler,
                                traverse_node_offset(node),
                                ImlValue::from(value),
                            );
                        }
                    }

                    ops.push(res);

                    // Push operation position here
                    ops.push(traverse_offset(node));

                    ImlOp::from(match parts[2] {
                        "not" => Op::UnaryOp("not"),
                        "neg" => Op::UnaryOp("neg"),
                        _ => {
                            unimplemented!("{}", emit);
                        }
                    })
                }

                "binary" | "logical" => {
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

                    match parts[2] {
                        "and" => {
                            ops.push(left);
                            ops.push(traverse_offset(node));

                            ImlOp::If {
                                peek: true,
                                test: true,
                                then: Box::new(right),
                                else_: Box::new(ImlOp::from(Op::Push)),
                            }
                        }
                        "or" => {
                            ops.push(left);
                            ops.push(traverse_offset(node));

                            ImlOp::If {
                                peek: true,
                                test: false,
                                then: Box::new(right),
                                else_: Box::new(ImlOp::from(Op::Push)),
                            }
                        }
                        _ => {
                            // When both operands are direct values, evaluate operation at compile-time
                            if let (Ok(left), Ok(right)) =
                                (left.get_evaluable_value(), right.get_evaluable_value())
                            {
                                if let Ok(value) = left.binary_op(right, parts[2]) {
                                    return ImlOp::load(
                                        compiler,
                                        traverse_node_offset(node),
                                        ImlValue::from(value),
                                    );
                                }
                            }

                            // Otherwise, generate code for operation
                            ops.push(left);
                            ops.push(right);

                            // Push operation position here
                            ops.push(traverse_offset(node));

                            ImlOp::from(match parts[2] {
                                "add" => Op::BinaryOp("add"),
                                "sub" => Op::BinaryOp("sub"),
                                "mul" => Op::BinaryOp("mul"),
                                "div" => Op::BinaryOp("div"),
                                "divi" => Op::BinaryOp("divi"),
                                "mod" => Op::BinaryOp("mod"),
                                _ => {
                                    unimplemented!("{}", emit);
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
                                if matches!(res, ImlOp::Load { .. } | ImlOp::Call { .. }) {
                                    "value"
                                } else {
                                    "sequence"
                                }
                            ),
                        ));
                    } else {
                        compiler.parselet_mark_consuming();
                    }

                    // Modifiers on usages of Token::Char can be optimized for better efficiency
                    if let ImlOp::Call {
                        target: ImlValue::Value(target),
                        ..
                    } = &res
                    {
                        let target = target.borrow();

                        if let Some(Token::Char(ccl)) = target.object::<Token>() {
                            match parts[2] {
                                // mod_pos on Token::Char becomes Token::Chars
                                "pos" | "kle" => {
                                    let mut chars = ImlOp::call(
                                        compiler,
                                        traverse_node_offset(node),
                                        ImlValue::from(RefValue::from(Token::Chars(ccl.clone()))),
                                        None,
                                    );
                                    if parts[2] == "kle" {
                                        // mod_kle on Token::Char becomes Token::Chars.into_optional()
                                        chars = chars.into_optional();
                                    }

                                    return chars;
                                }

                                // mod_not on Token::Char becomes negated Token::Char
                                "not" => {
                                    return ImlOp::call(
                                        compiler,
                                        traverse_node_offset(node),
                                        ImlValue::from(RefValue::from(Token::Char(
                                            ccl.clone().negate(),
                                        ))),
                                        None,
                                    );
                                }
                                _ => {}
                            }
                        }
                    }

                    // Push operation position here
                    ops.push(traverse_offset(node));

                    match parts[2] {
                        "pos" => res.into_positive(),
                        "kle" => res.into_kleene(),
                        "opt" => res.into_optional(),
                        "peek" => res.into_peek(),
                        "expect" => res.into_expect(None),
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
                        ImlOp::from(Op::Push)
                    };

                    // Compile time evaluation;
                    // In case the condition of the if already fails here, it doesn't need to be
                    // compiled into the program.
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
                    assert!(children.len() == 3);

                    let (var, iter_expr, body) = (
                        &children[0].borrow(),
                        &children[1].borrow(),
                        &children[2].borrow(),
                    );

                    let var = var.object::<Dict>().unwrap();
                    let iter_expr = iter_expr.object::<Dict>().unwrap();
                    let body = body.object::<Dict>().unwrap();

                    let temp = compiler.pop_temp(); // Borrow a temporary, unnamed variable

                    // Create an iter() on the iter expression
                    let initial = ImlOp::from(vec![
                        traverse_node_rvalue(compiler, iter_expr, Rvalue::CallOrLoad),
                        {
                            let iter = compiler.get_builtin("iter").unwrap();
                            ImlOp::call(compiler, None, iter, Some((1, false)))
                        },
                        ImlOp::from(Op::StoreFast(temp)),
                    ]);

                    // Create the condition, which calls iter_next() until void is returned
                    let condition = ImlOp::from(vec![
                        ImlOp::from(Op::LoadFast(temp)),
                        {
                            let iter_next = compiler.get_builtin("iter_next").unwrap();
                            ImlOp::call(compiler, None, iter_next, Some((1, false)))
                        },
                        traverse_node_lvalue(
                            compiler, var, true, true, //hold for preceding loop break check
                        ),
                    ]);

                    compiler.loop_push();
                    let body = traverse_node_rvalue(compiler, body, Rvalue::Load);
                    compiler.loop_pop();
                    compiler.push_temp(temp); // Give temp variable back for possible reuse.

                    ImlOp::Loop {
                        iterator: true,
                        initial: Box::new(initial),
                        condition: Box::new(condition),
                        body: Box::new(body),
                    }
                }

                "loop" => {
                    let children = List::from(&node["children"]);

                    compiler.loop_push();

                    let ret = match children.len() {
                        1 => {
                            let body = &children[0].borrow();

                            ImlOp::Loop {
                                iterator: false,
                                initial: Box::new(ImlOp::Nop),
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
                                iterator: false,
                                initial: Box::new(ImlOp::Nop),
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

                    compiler.loop_pop();

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
        "sequence" | "inline_sequence" | "list" => {
            let children = if let Some(children) = node.get_str("children") {
                List::from(children)
            } else {
                List::new()
            };

            let mut ops = Vec::new();

            for node in children.iter() {
                ops.push(traverse_node_rvalue(
                    compiler,
                    node.borrow().object::<Dict>().unwrap(),
                    Rvalue::CallOrLoad, // fixme: only statics or "real" rvalue nodes should be called, others just loaded
                ));
            }

            // Lists are definitive lists with a given length and only non-aliased values
            if emit == "list" {
                ops.push(Op::MakeList(children.len()).into());
                ImlOp::from(ops)
            }
            // In most cases, lists are parsed as sequences;
            else {
                ImlOp::seq(ops, true)
            }
        }

        // ---------------------------------------------------------------
        _ => unreachable!("No handling for {:?}", emit),
    }
}

/// Debug function to print an AST to stdout.
pub fn print(ast: &RefValue) {
    fn print(value: &RefValue, indent: usize) {
        let value = value.borrow();

        if let Some(d) = value.object::<Dict>() {
            let emit = d["emit"].to_string();

            let row = d
                .get_str("row")
                .and_then(|row| Some(row.borrow().to_usize().unwrap()));
            let col = d
                .get_str("col")
                .and_then(|col| Some(col.borrow().to_usize().unwrap()));
            let stop_row = d
                .get_str("stop_row")
                .and_then(|row| Some(row.borrow().to_usize().unwrap()));
            let stop_col = d
                .get_str("stop_col")
                .and_then(|col| Some(col.borrow().to_usize().unwrap()));

            let value = d.get_str("value");
            let children = d.get_str("children");

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

tokay_function!("ast : @emit, value=void, flatten=true, debug=false", {
    let context = context.unwrap();

    let mut ret = Dict::new();
    ret.insert_str("emit", emit.clone());

    // Need current frame position
    let capture_start = context.frame.capture_start;

    let value = if value.is_void() {
        Some(
            context
                .collect(capture_start, false, debug.is_true())
                .extract(&context.runtime.reader),
        )
    } else {
        Some(value)
    };

    // Debug
    if debug.is_true() {
        println!(
            "ast(emit={:?}, value={:?}, flatten={:?})",
            emit, value, flatten
        );
    }

    if let Some(value) = value {
        // Lists can be flattened
        if value.borrow().object::<List>().is_some() {
            ret.insert_str(
                "children",
                if flatten.is_true() {
                    List::list_flatten(vec![value], None).unwrap()
                } else {
                    value.clone()
                },
            );
        }
        // Dicts can be used directly as children
        else if value.borrow().object::<Dict>().is_some() {
            ret.insert_str("children", value.clone());
        }
        // Otherwise this is a value
        else {
            ret.insert_str("value", value.clone());
        }
    }

    let start = context.frame.reader_start;
    let reader_start = context.runtime.reader.start();

    // Store positions of reader start
    ret.insert_str("offset", value!(start.offset + reader_start.offset));
    ret.insert_str("row", value!(start.row as usize));
    ret.insert_str("col", value!(start.col as usize));

    // Store positions of reader stop
    let current = context.runtime.reader.tell();

    ret.insert_str("stop_offset", value!(current.offset + reader_start.offset));
    ret.insert_str("stop_row", value!(current.row as usize));
    ret.insert_str("stop_col", value!(current.col as usize));

    RefValue::from(ret).into()
});

tokay_function!("ast_print : @ast", {
    print(&ast);
    value!(void).into()
});

tokay_function!("ast2rust : @ast, level=0", {
    fn print(value: &RefValue, indent: usize, first: bool) {
        let (br_left, br_right) = if first { ("", "") } else { ("(", ")") };
        let value = value.borrow();

        if let Some(d) = value.object::<Dict>() {
            let emit = d["emit"].to_string();
            let value = d.get_str("value");
            let children = d.get_str("children");

            print!(
                "{space:indent$}{br_left}value!([\n{space:indent$}    \"emit\" => {emit:?}",
                space = "",
                indent = indent * 4,
                br_left = br_left,
                emit = emit
            );

            if let Some(children) = children {
                print!(
                    ",\n{space:indent$}    \"children\" =>\n",
                    space = "",
                    indent = indent * 4
                );

                print(children, indent + 2, false);
            }

            if let Some(value) = value {
                print!(
                    ",\n{space:indent$}    \"value\" => ",
                    space = "",
                    indent = indent * 4
                );
                print(value, indent, false);
            }

            print!(
                "\n{space:indent$}]){br_right}",
                space = "",
                indent = indent * 4,
                br_right = br_right
            );
        } else if let Some(l) = value.object::<List>() {
            print!(
                "{space:indent$}{br_left}value!([\n",
                space = "",
                indent = indent * 4,
                br_left = br_left
            );

            let mut iter = l.iter().peekable();

            while let Some(item) = iter.next() {
                print(item, indent + 1, false);
                if iter.peek().is_some() {
                    print!(",\n");
                }
            }

            print!(
                "\n{space:indent$}]){br_right}",
                space = "",
                indent = indent * 4,
                br_right = br_right
            );
        } else {
            assert!(
                ["str", "int", "float", "bool"].contains(&value.name()),
                "No matching Rust primitive for {} found",
                value.name()
            );

            // Rust primitives are mostly equal to Tokay's repr
            print!("{}", value.repr());
        }
    }

    print(&ast, level.to_usize()?, true);
    print!("\n");
    value!(void).into()
});
