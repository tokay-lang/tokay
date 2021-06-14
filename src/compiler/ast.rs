//! Compiler's internal Abstract Syntax Tree traversal

use std::collections::HashSet;

use super::*;
use crate::ccl::Ccl;
use crate::error::Error;
use crate::reader::Offset;
use crate::token::Token;
use crate::utils;
use crate::value::{BorrowByIdx, BorrowByKey, Dict, RefValue, Value};
use crate::vm::*;

/** AST traversal result.

This enum is used to allow either for a value or ops created during the AST traversal in the compiler.
*/
#[derive(Debug)]
enum AstResult {
    Empty,
    Value(RefValue),
    Identifier(String, Option<Offset>),
    Ops(Vec<Op>),
}

impl AstResult {
    /** Turns a traversal result into a vector of operations;

    In case the result is a Value, it can either be called when calling with 0 arguments is possible,
    which is specified by the call flag.
    */
    fn into_ops(self, compiler: &mut Compiler, call: bool) -> Vec<Op> {
        match self {
            AstResult::Empty => Vec::new(),
            AstResult::Value(value) => {
                let inner = value.borrow();

                vec![if call && inner.is_callable(false) {
                    if let Value::Token(_) = &*inner {
                        compiler.scopes[0].consuming = true;
                    }

                    Op::CallStatic(compiler.define_static(value.clone()))
                } else {
                    // void, true and false can be directly pushed
                    match &*inner {
                        Value::Integer(0) => Op::Push0,
                        Value::Integer(1) => Op::Push1,
                        Value::Void => Op::PushVoid,
                        Value::Null => Op::PushNull,
                        Value::True => Op::PushTrue,
                        Value::False => Op::PushFalse,
                        _ => Op::LoadStatic(compiler.define_static(value.clone())),
                    }
                }]
            }
            AstResult::Identifier(name, offset) => {
                // In case there is a use of a known constant,
                // directly return its value as AstResult.
                if let Some(value) = compiler.get_constant(&name) {
                    AstResult::Value(value).into_ops(compiler, call)
                } else {
                    let usage = if call {
                        Usage::CallOrCopy { name, offset }
                    } else {
                        Usage::Load { name, offset }
                    };

                    usage.resolve_or_dispose(compiler)
                }
            }
            AstResult::Ops(ops) => ops,
        }
    }

    /** Returns a value to operate with or evaluate during compile-time.

    The function will only return Ok(RefValue) when static_expression_evaluation-feature
    is enabled, the AstResult contains a value and this value is NOT a callable! */
    fn get_evaluable_value(&self) -> Result<RefValue, ()> {
        if cfg!(feature = "static_expression_evaluation") {
            if let AstResult::Value(value) = self {
                if !value.borrow().is_callable(false) {
                    return Ok(value.clone());
                }
            }
        }

        Err(())
    }
}

/// Checks whether identifier's name is the name of a reserved word.
pub(crate) fn identifier_is_valid(ident: &str) -> Result<(), Error> {
    match ident {
        "accept" | "begin" | "else" | "end" | "expect" | "false" | "for" | "if" | "in" | "not"
        | "null" | "peek" | "reject" | "return" | "true" | "void" | "while" => Err(Error::new(
            None,
            format!("Expected identifier, found reserved word '{}'", ident),
        )),
        _ => Ok(()),
    }
}

/// Checks whether identifier's name defines a consumable.
pub(crate) fn identifier_is_consumable(ident: &str) -> bool {
    let ch = ident.chars().next().unwrap();
    ch.is_uppercase() || ch == '_'
}

/// AST traversal entry
pub(super) fn traverse(compiler: &mut Compiler, ast: &Value) {
    traverse_node_or_list(compiler, ast);
}

// Traverse either a node or a list from the AST
fn traverse_node_or_list(compiler: &mut Compiler, ast: &Value) -> AstResult {
    if let Some(list) = ast.get_list() {
        let mut ops = Vec::new();

        for item in list.iter() {
            match traverse_node_or_list(compiler, &item.borrow()) {
                AstResult::Empty => {}
                AstResult::Ops(oplist) => ops.extend(oplist),
                _ => unreachable!("{:#?} cannot be handled", list),
            }
        }

        if ops.len() > 0 {
            AstResult::Ops(ops)
        } else {
            AstResult::Empty
        }
    } else if let Some(dict) = ast.get_dict() {
        traverse_node(compiler, dict)
    } else {
        AstResult::Value(ast.clone().into_refvalue())
    }
}

// Extract offset positions into an Offset structure
fn traverse_node_offset(node: &Dict) -> Option<Offset> {
    let offset = node
        .get("offset")
        .and_then(|offset| Some(offset.borrow().to_addr()));
    let row = node
        .get("row")
        .and_then(|row| Some(row.borrow().to_addr() as u32));
    let col = node
        .get("col")
        .and_then(|col| Some(col.borrow().to_addr() as u32));

    if let (Some(offset), Some(row), Some(col)) = (offset, row, col) {
        Some(Offset { offset, row, col })
    } else {
        None
    }
}

// Traverse a value node into a Value instance
fn traverse_node_value(compiler: &mut Compiler, node: &Dict) -> Value {
    let emit = node.borrow_by_key("emit");
    let emit = emit.get_string().unwrap();

    // Generate a value from the given code
    match emit {
        // Literals
        "value_string" => {
            let value = node.borrow_by_key("value").to_string();
            Value::String(value)
        }
        "value_integer" => {
            let value = node.borrow_by_key("value").to_string();
            Value::Integer(match value.parse::<i64>() {
                Ok(i) => i,
                Err(_) => 0,
            })
        }
        "value_float" => {
            let value = node.borrow_by_key("value").to_string();
            Value::Float(match value.parse::<f64>() {
                Ok(f) => f,
                Err(_) => 0.0,
            })
        }
        "value_true" => Value::True,
        "value_false" => Value::False,
        "value_null" => Value::Null,
        "value_void" => Value::Void,

        // Tokens
        "value_token_match" | "value_token_touch" => {
            let value = utils::unescape(node.borrow_by_key("value").to_string());

            if emit == "value_token_match" {
                Token::Match(value).into_value()
            } else {
                Token::Touch(value).into_value()
            }
        }
        "value_token_any" => Token::Any.into_value(),
        "value_token_ccl" => {
            let node = node.borrow_by_key("children").to_dict();

            let emit = node.borrow_by_key("emit");
            let emit = emit.get_string().unwrap();

            let children = node.borrow_by_key("children").to_list();

            let mut ccl = Ccl::new();

            for range in children {
                let range = range.borrow().to_dict();

                let emit = range.borrow_by_key("emit");
                let emit = emit.get_string().unwrap();

                let value = range.borrow_by_key("value");
                let value = value.get_string().unwrap();

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
                ccl.negate();
            } else {
                assert!(emit == "ccl");
            }

            Token::Char(ccl).into_value()
        }

        // Parselets
        "value_parselet" => {
            compiler.push_scope(true);

            let children = node.borrow_by_key("children");

            let (args, body) = if let Some(children) = children.get_list() {
                assert!(children.len() == 2);
                (Some(children[0].borrow()), children[1].borrow())
            } else {
                (None, children)
            };

            // Create signature
            let mut sig: Vec<(String, Option<usize>)> = Vec::new();
            let mut sig_names = HashSet::new();

            if let Some(args) = args {
                for node in args.to_list() {
                    let node = node.borrow();
                    let node = node.get_dict().unwrap();

                    let children = node.borrow_by_key("children").to_list();

                    let ident = children.borrow_by_idx(0);
                    let ident = ident.get_dict().unwrap().borrow_by_key("value").to_string();

                    // Check for correct identifier semantics
                    if !ident.chars().nth(0).unwrap().is_lowercase() {
                        compiler.errors.push(
                            Error::new(
                                traverse_node_offset(node),
                                format!("Variable identifier '{}' invalid; Use identifier starting in lower-case, e.g. '{}{}'",
                                ident, &ident[0..1].to_lowercase(), &ident[1..])
                            )
                        );
                    }

                    // check if identifier was not provided twice
                    if sig_names.contains(&ident) {
                        compiler.errors.push(Error::new(
                            traverse_node_offset(node),
                            format!("Identifier '{}' already given in signature before", ident),
                        ));

                        continue;
                    } else {
                        sig_names.insert(ident.clone());
                    }

                    compiler.new_local(&ident);

                    assert!(children.len() <= 2);
                    let default = if children.len() == 2 {
                        let default = children.borrow_by_idx(1);
                        let value =
                            traverse_node_static(compiler, None, default.get_dict().unwrap());
                        Some(compiler.define_static(value))
                    } else {
                        None
                    };

                    sig.push((ident.clone(), default));
                    //println!("{} {} {:?}", emit.to_string(), ident, default);
                }
            }

            //println!("sig = {:?}", sig);

            // Body
            let body = traverse_node(compiler, &body.get_dict().unwrap());
            let body = Op::from_vec(body.into_ops(compiler, true));

            compiler
                .create_parselet(None, sig, body, None, false, false)
                .into_value()
        }
        _ => unimplemented!("unhandled value node {}", emit),
    }
}

// Traverse a static value
fn traverse_node_static(compiler: &mut Compiler, name: Option<String>, node: &Dict) -> RefValue {
    compiler.push_scope(true);

    match traverse_node(compiler, node) {
        AstResult::Empty => {
            compiler.pop_scope();
            Value::Void.into_refvalue()
        }
        AstResult::Value(value) => {
            compiler.pop_scope();

            if name.is_some() {
                if let Value::Parselet(parselet) = &*value.borrow() {
                    let mut parselet = parselet.borrow_mut();
                    parselet.name = name;
                }
            }

            value
        }
        AstResult::Ops(ops) => compiler
            .create_parselet(name, Vec::new(), Op::from_vec(ops), None, false, false)
            .into_value()
            .into_refvalue(),
        _ => unreachable!(),
    }
}

// Traverse lvalue
fn traverse_node_lvalue(
    compiler: &mut Compiler,
    node: &Dict,
    store: bool,
    hold: bool,
) -> AstResult {
    let children = node.borrow_by_key("children").to_list();

    let mut ops = Vec::new();

    for (i, item) in children.iter().enumerate() {
        let item = item.borrow();
        let item = item.get_dict().unwrap();

        let emit = item.borrow_by_key("emit");
        let emit = emit.get_string().unwrap();

        match emit {
            capture if capture.starts_with("capture") => {
                let children = item.borrow_by_key("children");

                match capture {
                    "capture_expr" => {
                        ops.extend(
                            traverse_node_or_list(compiler, &children).into_ops(compiler, false),
                        );
                        if store && hold {
                            ops.push(Op::StoreCaptureHold)
                        } else if store {
                            ops.push(Op::StoreCapture)
                        } else {
                            ops.push(Op::LoadCapture)
                        }
                    }

                    "capture_index" => {
                        let children = children.get_dict().unwrap();
                        let index = traverse_node_value(compiler, children);

                        if store && hold {
                            ops.push(Op::StoreFastCaptureHold(index.to_addr()));
                        } else if store {
                            ops.push(Op::StoreFastCapture(index.to_addr()));
                        } else {
                            ops.push(Op::LoadFastCapture(index.to_addr()));
                        }
                    }

                    "capture_alias" => {
                        unimplemented!("//todo");
                    }

                    _ => {
                        unreachable!();
                    }
                }
            }

            "identifier" => {
                let name = item.borrow_by_key("value");
                let name = name.get_string().unwrap();

                // Check for not assigning to a constant (at any level)
                if compiler.get_constant(name).is_some() {
                    compiler.errors.push(Error::new(
                        traverse_node_offset(node),
                        format!("Cannot assign to constant '{}'", name),
                    ));

                    break;
                }

                if i < children.len() - 1 {
                    ops.extend(
                        Usage::Load {
                            name: name.to_string(),
                            offset: traverse_node_offset(item),
                        }
                        .resolve_or_dispose(compiler),
                    )
                } else {
                    // Check if identifier is valid
                    if let Err(mut error) = identifier_is_valid(name) {
                        if let Some(offset) = traverse_node_offset(node) {
                            error.patch_offset(offset);
                        }

                        compiler.errors.push(error);
                        break;
                    }

                    // Check if identifier is not defining a consumable
                    if identifier_is_consumable(name) {
                        compiler.errors.push(Error::new(
                            traverse_node_offset(node),
                            format!(
                                "Cannot assign variable named '{}'; Try lower-case identifier, e.g. '{}'",
                                name, name.to_lowercase()
                            ),
                        ));

                        break;
                    }

                    ops.push(
                        /* Generates code for a symbol store, which means:

                            1. look-up local variable, and store into
                            2. look-up global variable, and store into
                            3. create local variable, and store into
                        */
                        if let Some(addr) = compiler.get_local(name) {
                            if store && hold {
                                Op::StoreFastHold(addr)
                            } else if store {
                                Op::StoreFast(addr)
                            } else {
                                Op::LoadFast(addr)
                            }
                        } else if let Some(addr) = compiler.get_global(name) {
                            if store && hold {
                                Op::StoreGlobalHold(addr)
                            } else if store {
                                Op::StoreGlobal(addr)
                            } else {
                                Op::LoadGlobal(addr)
                            }
                        } else {
                            let addr = compiler.new_local(name);
                            if store && hold {
                                Op::StoreFastHold(addr)
                            } else if store {
                                Op::StoreFast(addr)
                            } else {
                                Op::LoadFast(addr)
                            }
                        },
                    )
                }
            }

            other => {
                unimplemented!("{:?} not implemented for lvalue", other);
            }
        }
    }

    AstResult::Ops(ops)
}

// Main traversal function, running recursively through the AST
fn traverse_node(compiler: &mut Compiler, node: &Dict) -> AstResult {
    // Normal node processing...
    let emit = node.borrow_by_key("emit");
    let emit = emit.get_string().unwrap();

    //println!("emit = {:?}", emit);

    match emit {
        "alias" => {
            let children = node.borrow_by_key("children");
            let children = children.get_list().unwrap();
            assert_eq!(children.len(), 2);

            let (left, right) = children.borrow_first_2();

            let left = traverse_node(compiler, &left.get_dict().unwrap());
            let right = traverse_node(compiler, &right.get_dict().unwrap());

            // Push value first, then the alias
            let mut ops = right.into_ops(compiler, true);
            ops.extend(left.into_ops(compiler, true));
            ops.push(Op::MakeAlias);

            AstResult::Ops(ops)
        }

        // assign ---------------------------------------------------------
        assign if assign.starts_with("assign") => {
            let children = node.borrow_by_key("children");
            let children = children.get_list();

            let (lvalue, value) = children.unwrap().borrow_first_2();
            let lvalue = lvalue.get_dict().unwrap();
            let value = value.get_dict().unwrap();

            let parts: Vec<&str> = assign.split("_").collect();

            let mut ops = Vec::new();

            if parts.len() > 1 && parts[1] != "hold" {
                ops.extend(
                    traverse_node_lvalue(compiler, lvalue, false, false).into_ops(compiler, false),
                );
                ops.extend(traverse_node(compiler, value).into_ops(compiler, false));

                match parts[1] {
                    "add" => ops.push(Op::InlineAdd),
                    "sub" => ops.push(Op::InlineSub),
                    "mul" => ops.push(Op::InlineMul),
                    "div" => ops.push(Op::InlineDiv),
                    _ => unreachable!(),
                }

                if *parts.last().unwrap() != "hold" {
                    ops.push(Op::Drop);
                }
            } else {
                ops.extend(traverse_node(compiler, value).into_ops(compiler, false));
                ops.extend(
                    traverse_node_lvalue(compiler, lvalue, true, *parts.last().unwrap() == "hold")
                        .into_ops(compiler, false),
                );
            }

            AstResult::Ops(ops)
        }

        // attribute ------------------------------------------------------
        "attribute" => {
            unimplemented!();

            //let mut ops = traverse_node_or_list(compiler, &node.borrow_by_key("children")).into_ops(compiler, true);
            //ops.push(Op::LoadAttr);

            //AstResult::Ops(ops)
        }
        // begin ----------------------------------------------------------
        "begin" | "end" => {
            if compiler.scopes[0].variables.is_none() {
                compiler.errors.push(Error::new(
                    traverse_node_offset(node),
                    format!("'{}' may only be used in parselet scope", emit),
                ))
            }

            let ops = traverse_node_or_list(compiler, &node.borrow_by_key("children"))
                .into_ops(compiler, true);

            if emit == "begin" {
                compiler.scopes[0].begin.extend(ops);
            } else {
                compiler.scopes[0].end.extend(ops);
            }

            AstResult::Empty
        }

        // block ----------------------------------------------------------
        "block" => {
            if let Some(children) = node.get("children") {
                let body =
                    traverse_node_or_list(compiler, &children.borrow()).into_ops(compiler, true);
                AstResult::Ops(vec![Block::new(body)])
            } else {
                AstResult::Empty
            }
        }

        // call -----------------------------------------------------------
        "call" => {
            let children = node.borrow_by_key("children");
            let children = children.to_list();

            let mut ops = Vec::new();
            let mut args = 0;
            let mut nargs = 0;

            if children.len() > 1 {
                let params = children[1].borrow().to_list();

                for param in &params {
                    let param = param.borrow();
                    let param = param.get_dict().unwrap();

                    let emit = param.borrow_by_key("emit");

                    match emit.get_string().unwrap() {
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

                            ops.extend(
                                traverse_node_or_list(compiler, &param.borrow_by_key("children"))
                                    .into_ops(compiler, false),
                            );
                            args += 1;
                        }

                        "param_named" => {
                            let children = param.borrow_by_key("children").to_list();

                            ops.extend(
                                traverse_node_or_list(compiler, &children.borrow_by_idx(1))
                                    .into_ops(compiler, false),
                            );

                            let ident = children.borrow_by_idx(0);
                            let ident =
                                ident.get_dict().unwrap().borrow_by_key("value").to_string();
                            ops.push(Op::LoadStatic(
                                compiler.define_static(Value::String(ident).into_refvalue()),
                            ));

                            nargs += 1;
                        }

                        other => unimplemented!("Unhandled parameter type {:?}", other),
                    }
                }
            }

            // When calling with nargs, create a nargs dict first
            if nargs > 0 {
                ops.push(Op::MakeDict(nargs));
            }

            // Push call position here
            if let Some(offset) = traverse_node_offset(node) {
                ops.push(Op::Offset(Box::new(offset)));
            }

            // Perform static call or resolved rvalue call
            let callee = traverse_node_or_list(compiler, &children[0].borrow());

            if let AstResult::Identifier(ident, offset) = callee {
                if identifier_is_consumable(&ident) {
                    compiler.scopes[0].consuming = true;
                }

                ops.extend(
                    Usage::Call {
                        name: ident.to_string(),
                        args,
                        nargs,
                        offset,
                    }
                    .resolve_or_dispose(compiler),
                )
            } else {
                ops.extend(callee.into_ops(compiler, false));

                if args == 0 && nargs == 0 {
                    ops.push(Op::Call);
                } else if args > 0 && nargs == 0 {
                    ops.push(Op::CallArg(args));
                } else {
                    ops.push(Op::CallArgNamed(args))
                }
            }

            AstResult::Ops(ops)
        }

        // capture --------------------------------------------------------
        "capture_alias" | "capture_expr" => {
            let children = node.borrow_by_key("children");

            let mut ops = traverse_node_or_list(compiler, &children).into_ops(compiler, false);
            ops.push(Op::LoadCapture);
            AstResult::Ops(ops)
        }

        "capture_index" => {
            let children = node.borrow_by_key("children");

            let children = children.get_dict().unwrap();
            let index = traverse_node_value(compiler, children);
            AstResult::Ops(vec![Op::LoadFastCapture(index.to_addr())])
        }

        // constant -------------------------------------------------------
        "constant" => {
            let children = node.borrow_by_key("children");
            let children = children.get_list();

            let (ident, value) = children.unwrap().borrow_first_2();

            let ident = ident.get_dict().unwrap();
            let ident = ident.borrow_by_key("value");
            let ident = ident.get_string().unwrap();

            if let Err(mut error) = identifier_is_valid(ident) {
                if let Some(offset) = traverse_node_offset(node) {
                    error.patch_offset(offset);
                }

                compiler.errors.push(error);
                return AstResult::Empty;
            }

            // Distinguish between pure values or an expression
            let node = value.get_dict().unwrap();

            // fixme: Restricted to pure values currently.
            let value = traverse_node_static(compiler, Some(ident.to_string()), node);

            if value.borrow().is_consuming() {
                if !identifier_is_consumable(ident) {
                    compiler.errors.push(Error::new(
                        traverse_node_offset(node),
                        format!(
                            "Cannot assign constant '{}' as consumable. Use identifier starting in upper-case, e.g. '{}{}'",
                            ident, &ident[0..1].to_uppercase(), &ident[1..]
                        )
                    ));
                }
            } else if identifier_is_consumable(ident) {
                compiler.errors.push(Error::new(
                    traverse_node_offset(node),
                    format!(
                        "Cannot assign to constant '{}'. Use identifier starting in lower-case, e.g. '{}{}'",
                        ident, &ident[0..1].to_lowercase(), &ident[1..]
                    ),
                ));
            }

            //println!("{} : {:?}", ident, value);
            compiler.set_constant(ident, value);

            // Try to resolve usage of newly introduced constant in current scope
            compiler.resolve_scope();

            AstResult::Empty
        }

        // identifier -----------------------------------------------------
        "identifier" => {
            let name = node.borrow_by_key("value").to_string();
            AstResult::Identifier(name, traverse_node_offset(node))
        }

        // index ----------------------------------------------------------
        "index" => {
            let mut ops = traverse_node_or_list(compiler, &node.borrow_by_key("children"))
                .into_ops(compiler, true);
            ops.push(Op::LoadIndex); // todo: in case value is an integer, use LoadFastIndex
            AstResult::Ops(ops)
        }

        // inplace --------------------------------------------------------
        inplace if inplace.starts_with("inplace_") => {
            let children = node.borrow_by_key("children");
            let lvalue = children.get_dict().unwrap();

            let mut ops = Vec::new();

            ops.extend(
                traverse_node_lvalue(compiler, lvalue, false, false).into_ops(compiler, false),
            );

            let parts: Vec<&str> = inplace.split("_").collect();

            match parts[1] {
                "pre" => {
                    ops.push(if parts[2] == "inc" {
                        Op::InlineInc
                    } else {
                        Op::InlineDec
                    });
                }
                "post" => {
                    ops.extend(vec![
                        Op::Dup,
                        Op::Rot2,
                        if parts[2] == "inc" {
                            Op::InlineInc
                        } else {
                            Op::InlineDec
                        },
                        Op::Drop,
                    ]);
                }
                _ => unreachable!(),
            }

            AstResult::Ops(ops)
        }

        // main -----------------------------------------------------------
        "main" => {
            let children = node.borrow_by_key("children");

            let body = traverse_node_or_list(compiler, &children).into_ops(compiler, true);
            let main = compiler.create_parselet(
                Some("__main__".to_string()),
                Vec::new(),
                if body.len() > 0 {
                    Block::new(body)
                } else {
                    Op::Nop
                },
                None,
                false,
                true,
            );

            compiler.define_static(main.into_value().into_refvalue());
            AstResult::Empty
        }

        // operator ------------------------------------------------------
        op if op.starts_with("op_") => {
            let parts: Vec<&str> = emit.split("_").collect();
            let mut ops = Vec::new();

            let op = match parts[1] {
                "accept" | "repeat" => {
                    let children = node.borrow_by_key("children");

                    match traverse_node(compiler, &children.get_dict().unwrap()) {
                        AstResult::Value(value) if matches!(&*value.borrow(), Value::Void) => {
                            if parts[1] == "repeat" {
                                Op::Repeat
                            } else {
                                Op::Accept
                            }
                        }
                        result => {
                            ops.extend(result.into_ops(compiler, false));
                            if parts[1] == "repeat" {
                                Op::LoadRepeat
                            } else {
                                Op::LoadAccept
                            }
                        }
                    }
                }

                "binary" => {
                    let children = node.borrow_by_key("children");
                    let children = children.get_list().unwrap();
                    assert_eq!(children.len(), 2);

                    let (left, right) = children.borrow_first_2();

                    let left = traverse_node(compiler, &left.get_dict().unwrap());
                    let right = traverse_node(compiler, &right.get_dict().unwrap());

                    // When both results are values, calculate in-place
                    if let (Ok(left), Ok(right)) =
                        (left.get_evaluable_value(), right.get_evaluable_value())
                    {
                        return AstResult::Value(
                            match parts[2] {
                                "add" => left.borrow().add(&*right.borrow()).unwrap(),
                                "sub" => left.borrow().sub(&*right.borrow()).unwrap(),
                                "mul" => left.borrow().mul(&*right.borrow()).unwrap(),
                                "div" => left.borrow().div(&*right.borrow()).unwrap(),
                                _ => {
                                    unimplemented!("op_binary_{}", parts[2]);
                                }
                            }
                            .into_refvalue(),
                        );
                    }

                    // Otherwise, generate operational code
                    ops.extend(left.into_ops(compiler, true));
                    ops.extend(right.into_ops(compiler, true));

                    match parts[2] {
                        "add" => Op::Add,
                        "sub" => Op::Sub,
                        "mul" => Op::Mul,
                        "div" => Op::Div,
                        _ => {
                            unimplemented!("op_binary_{}", parts[2]);
                        }
                    }
                }

                "unary" => {
                    let children = node.borrow_by_key("children");
                    let children = children.get_dict().unwrap();

                    let res = traverse_node(compiler, children);
                    if let Ok(value) = res.get_evaluable_value() {
                        return AstResult::Value(
                            match parts[2] {
                                "not" => value.borrow().not().unwrap(),
                                "neg" => value.borrow().neg().unwrap(),
                                _ => {
                                    unimplemented!("op_unary_{}", parts[2]);
                                }
                            }
                            .into_refvalue(),
                        );
                    }

                    ops.extend(res.into_ops(compiler, true));

                    match parts[2] {
                        "not" => Op::Not,
                        "neg" => Op::Neg,
                        _ => {
                            unimplemented!("op_unary_{}", parts[2]);
                        }
                    }
                }

                "compare" | "logical" => {
                    let children = node.borrow_by_key("children");
                    let children = children.get_list().unwrap();
                    assert_eq!(children.len(), 2);

                    let (left, right) = children.borrow_first_2();
                    let left = traverse_node(compiler, &left.get_dict().unwrap());
                    let right = traverse_node(compiler, &right.get_dict().unwrap());

                    // When both results are values, compare in-place
                    if let (Ok(left), Ok(right)) =
                        (left.get_evaluable_value(), right.get_evaluable_value())
                    {
                        return AstResult::Value(
                            if match parts[2] {
                                "equal" => &*left.borrow() == &*right.borrow(),
                                "unequal" => &*left.borrow() != &*right.borrow(),
                                "lowerequal" => &*left.borrow() <= &*right.borrow(),
                                "greaterequal" => &*left.borrow() >= &*right.borrow(),
                                "lower" => &*left.borrow() < &*right.borrow(),
                                "greater" => &*left.borrow() > &*right.borrow(),
                                "and" => left.borrow().is_true() && right.borrow().is_true(),
                                "or" => left.borrow().is_true() || right.borrow().is_true(),
                                _ => {
                                    unimplemented!("op_compare_{}", parts[2]);
                                }
                            } {
                                Value::True.into_refvalue()
                            } else {
                                Value::False.into_refvalue()
                            },
                        );
                    }

                    // Otherwise, generate operational code
                    ops.extend(left.into_ops(compiler, false));

                    match parts[2] {
                        "and" => {
                            Op::IfTrue(Box::new(Op::from_vec(right.into_ops(compiler, false))))
                        }
                        "or" => {
                            Op::IfFalse(Box::new(Op::from_vec(right.into_ops(compiler, false))))
                        }
                        _ => {
                            ops.extend(right.into_ops(compiler, false));
                            match parts[2] {
                                "equal" => Op::Equal,
                                "unequal" => Op::NotEqual,
                                "lowerequal" => Op::LowerEqual,
                                "greaterequal" => Op::GreaterEqual,
                                "lower" => Op::Lower,
                                "greater" => Op::Greater,
                                _ => {
                                    unimplemented!("op_compare_{}", parts[2]);
                                }
                            }
                        }
                    }
                }

                "mod" => {
                    let children = node.borrow_by_key("children");
                    let children = children.get_dict().unwrap();

                    let res = traverse_node(compiler, children);

                    // Special operations for Token::Char
                    if let AstResult::Value(value) = &res {
                        let value = value.borrow();

                        if !value.is_consuming() {
                            compiler.errors.push(Error::new(
                                traverse_node_offset(node),
                                format!(
                                    "Operator '{}' has no effect on non-consuming value",
                                    parts[2]
                                ),
                            ));
                        } else {
                            compiler.scopes[0].consuming = true;
                        }

                        if let Value::Token(token) = &*value {
                            if let Token::Char(mut ccl) = *token.clone() {
                                match parts[2] {
                                    // mod_pos on Token::Char becomes Token::Chars
                                    "pos" | "kle" => {
                                        let chars = AstResult::Value(
                                            Token::Chars(ccl).into_value().into_refvalue(),
                                        );

                                        if parts[2] == "pos" {
                                            return chars;
                                        }

                                        // mod_kle on Token::Char becomes Token::Chars.into_optional()
                                        return AstResult::Ops(vec![Op::from_vec(
                                            chars.into_ops(compiler, true),
                                        )
                                        .into_optional()]);
                                    }

                                    // mod_not on Token::Char becomes negated Token::Char
                                    "not" => {
                                        ccl.negate();
                                        return AstResult::Value(
                                            Token::Char(ccl).into_value().into_refvalue(),
                                        );
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }

                    let op = Op::from_vec(res.into_ops(compiler, true));

                    match parts[2] {
                        "pos" => op.into_positive(),
                        "kle" => op.into_kleene(),
                        "opt" => op.into_optional(),
                        "peek" => Peek::new(op).into_op(),
                        "expect" => Expect::new(op, None).into_op(),
                        "not" => Not::new(op).into_op(),
                        _ => unreachable!(),
                    }
                }

                "if" | "ifelse" => {
                    let children = node.borrow_by_key("children");
                    let children = children.get_list().unwrap();

                    let condition = traverse_node_or_list(compiler, &children[0].borrow());
                    let then = traverse_node_or_list(compiler, &children[1].borrow());
                    let eelse = if children.len() == 3 {
                        Some(traverse_node_or_list(compiler, &children[2].borrow()))
                    } else {
                        None
                    };

                    // Compile time evaluation; When the if fails, it doesn't need
                    // to be compiled into the program.
                    if let Ok(value) = condition.get_evaluable_value() {
                        if value.borrow().is_true() {
                            return then;
                        } else if let Some(eelse) = eelse {
                            return eelse;
                        }

                        return AstResult::Empty;
                    }

                    ops.extend(condition.into_ops(compiler, false));
                    Op::If(Box::new((
                        Op::from_vec(then.into_ops(compiler, true)),
                        eelse.and_then(|eelse| Some(Op::from_vec(eelse.into_ops(compiler, true)))),
                    )))
                }

                _ => {
                    unimplemented!("{} missing", op);
                }
            };
            ops.push(op);

            AstResult::Ops(ops)
        }

        // rvalue ---------------------------------------------------------
        "rvalue" => {
            let children = node.borrow_by_key("children");
            let children = children.to_list();

            let mut ops = Vec::new();

            for node in children.iter() {
                ops.extend(
                    traverse_node_or_list(compiler, &node.borrow()).into_ops(compiler, false),
                );
            }

            assert!(ops.len() > 0);
            AstResult::Ops(ops)
        }

        // sequence  ------------------------------------------------------
        "sequence" | "collection" => {
            let children = node.borrow_by_key("children");
            let children = children.to_list();

            let mut ops = Vec::new();

            for node in &children {
                ops.extend(traverse_node_or_list(compiler, &node.borrow()).into_ops(compiler, true))
            }

            if ops.len() > 0 {
                if emit == "collection" {
                    ops.push(Op::MakeCollection(children.len()));
                    AstResult::Ops(ops)
                } else {
                    AstResult::Ops(vec![Sequence::new(ops)])
                }
            } else {
                AstResult::Empty
            }
        }

        // value ---------------------------------------------------------
        value if value.starts_with("value_") => {
            AstResult::Value(traverse_node_value(compiler, node).into_refvalue())
        }

        // ---------------------------------------------------------------
        _ => {
            // When there are children, try to traverse_node_or_list recursively
            if let Some(children) = node.get("children") {
                traverse_node_or_list(compiler, &children.borrow())
            }
            // Otherwise, report unhandled node!
            else {
                unreachable!("No handling for {:?}", node);
            }
        }
    }
}

/// Debug function to print an AST to stdout.
pub fn print(ast: &Value) {
    fn print(value: &Value, indent: usize) {
        match value {
            Value::Dict(d) => {
                let emit = d["emit"].borrow();
                let emit = emit.get_string().unwrap();

                let row = d.get("row").and_then(|row| Some(row.borrow().to_addr()));
                let col = d.get("col").and_then(|col| Some(col.borrow().to_addr()));
                let stop_row = d
                    .get("stop_row")
                    .and_then(|row| Some(row.borrow().to_addr()));
                let stop_col = d
                    .get("stop_col")
                    .and_then(|col| Some(col.borrow().to_addr()));

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
                    print!(" {:?}", value.borrow());
                }
                print!("\n");

                if let Some(children) = children {
                    print(&children.borrow(), indent + 1);
                }
            }

            Value::List(l) => {
                for item in l.iter() {
                    print(&item.borrow(), indent);
                }
            }

            other => print!("{}", other.repr()),
        }
    }

    print(ast, 0);
}
