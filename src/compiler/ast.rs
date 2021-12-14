//! Compiler's internal Abstract Syntax Tree traversal
use std::cell::{Ref, RefMut};
use std::collections::HashSet;

use charclass::CharClass;
use linkme::distributed_slice;

use super::*;
use crate::builtin::{Builtin, BUILTINS};
use crate::error::Error;
use crate::reader::Offset;
use crate::utils;
use crate::value::{Dict, List, Token, Value};
use crate::vm::*;

// Helper trait for Dict
trait BorrowByKey {
    fn borrow_by_key(&self, key: &str) -> Ref<Value>;
    fn borrow_by_key_mut(&self, key: &str) -> RefMut<Value>;
}

impl BorrowByKey for Dict {
    fn borrow_by_key(&self, key: &str) -> Ref<Value> {
        let value = self.get(key).unwrap();
        value.borrow()
    }

    fn borrow_by_key_mut(&self, key: &str) -> RefMut<Value> {
        let value = self.get(key).unwrap();
        value.borrow_mut()
    }
}

// Helper trait for List
trait BorrowByIdx {
    fn borrow_by_idx(&self, idx: usize) -> Ref<Value>;
    fn borrow_by_idx_mut(&self, idx: usize) -> RefMut<Value>;

    fn borrow_first(&self) -> Ref<Value> {
        self.borrow_by_idx(0)
    }

    fn borrow_first_2(&self) -> (Ref<Value>, Ref<Value>) {
        let first = self.borrow_by_idx(0);
        let second = self.borrow_by_idx(1);

        (first, second)
    }

    fn borrow_first_3(&self) -> (Ref<Value>, Ref<Value>, Ref<Value>) {
        let first = self.borrow_by_idx(0);
        let second = self.borrow_by_idx(1);
        let third = self.borrow_by_idx(2);

        (first, second, third)
    }
}

impl BorrowByIdx for List {
    fn borrow_by_idx(&self, idx: usize) -> Ref<Value> {
        let value = self.get(idx).unwrap();
        value.borrow()
    }

    fn borrow_by_idx_mut(&self, idx: usize) -> RefMut<Value> {
        let value = self.get(idx).unwrap();
        value.borrow_mut()
    }
}

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
pub(super) fn traverse(compiler: &mut Compiler, ast: &Value) {
    traverse_node_or_list(compiler, ast);
}

// Traverse either a node or a list from the AST
fn traverse_node_or_list(compiler: &mut Compiler, ast: &Value) -> ImlResult {
    if let Some(list) = ast.get_list() {
        let mut ops = Vec::new();

        for item in list.iter() {
            ops.extend(traverse_node_or_list(compiler, &item.borrow()).into_ops(compiler, false));
        }

        if ops.len() > 0 {
            ImlResult::Ops(ops)
        } else {
            ImlResult::Empty
        }
    } else if let Some(dict) = ast.get_dict() {
        traverse_node(compiler, dict)
    } else {
        ImlResult::Value(ast.clone().into())
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

// Append offset to ops
fn insert_offset(ops: &mut Vec<ImlOp>, node: &Dict) {
    if let Some(offset) = traverse_node_offset(node) {
        ops.push(Op::Offset(Box::new(offset)).into());
    }
}

// Traverse a value node into a Value instance
fn traverse_node_value(compiler: &mut Compiler, node: &Dict) -> ImlValue {
    let emit = node.borrow_by_key("emit");
    let emit = emit.get_string().unwrap();

    // Generate a value from the given code
    match emit {
        // Literals
        "value_string" => Value::String(
            node.borrow_by_key("value")
                .get_string()
                .unwrap()
                .to_string(),
        )
        .into(),
        "value_integer" => {
            let value = node.borrow_by_key("value").to_string();
            Value::Integer(match value.parse::<i64>() {
                Ok(i) => i,
                Err(_) => 0,
            })
            .into()
        }
        "value_float" => {
            let value = node.borrow_by_key("value").to_string();
            Value::Float(match value.parse::<f64>() {
                Ok(f) => f,
                Err(_) => 0.0,
            })
            .into()
        }
        "value_true" => Value::True.into(),
        "value_false" => Value::False.into(),
        "value_null" => Value::Null.into(),
        "value_void" => Value::Void.into(),

        // Tokens
        "value_token_match" | "value_token_touch" => {
            let mut value = node
                .borrow_by_key("value")
                .get_string()
                .unwrap()
                .to_string();

            if value.len() == 0 {
                compiler.errors.push(Error::new(
                    traverse_node_offset(node),
                    format!("Empty match not allowed"),
                ));
                value = "#INVALID".to_string();
            }

            if emit == "value_token_match" {
                Token::Match(value).into_value().into()
            } else {
                Token::Touch(value).into_value().into()
            }
        }
        "value_token_any" => Token::any().into_value().into(),
        "value_token_ccl" => {
            let node = node.borrow_by_key("children").to_dict();

            let emit = node.borrow_by_key("emit");
            let emit = emit.get_string().unwrap();

            let children = node.borrow_by_key("children").to_list();

            let mut ccl = CharClass::new();

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
                Token::Char(ccl.negate()).into_value().into()
            } else {
                assert!(emit == "ccl");
                Token::Char(ccl).into_value().into()
            }
        }

        // Parselets
        "value_parselet" => {
            compiler.push_parselet();

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
                        let value = traverse_node_static(
                            compiler,
                            Some(&ident),
                            default.get_dict().unwrap(),
                        );
                        Some(compiler.define_value(value))
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
            let body = ImlOp::from_vec(body.into_ops(compiler, true));

            compiler.pop_parselet(None, sig, body).into()
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

    // ... because when case ImlResult::Ops is returned here,
    // it would be nice to have it in a separate scope.
    match traverse_node(compiler, node) {
        ImlResult::Empty => {
            compiler.pop_parselet(None, Vec::new(), ImlOp::from(Op::Nop));
            Value::Void.into()
        }
        ImlResult::Value(value) => {
            compiler.pop_parselet(None, Vec::new(), ImlOp::from(Op::Nop));

            if let Some(lvalue) = lvalue {
                if let ImlValue::Parselet(parselet) = &value {
                    let mut parselet = parselet.borrow_mut();
                    parselet.name = Some(lvalue.to_string());
                }
            }

            value
        }
        other => {
            let ops = match other {
                ImlResult::Ops(ops) => ops,
                other => other.into_ops(compiler, false),
            };

            compiler
                .pop_parselet(
                    lvalue.and_then(|lvalue| Some(lvalue.to_string())),
                    Vec::new(),
                    ImlOp::from_vec(ops),
                )
                .into()
        }
    }
}

// Traverse lvalue
fn traverse_node_lvalue(
    compiler: &mut Compiler,
    node: &Dict,
    store: bool,
    hold: bool,
) -> ImlResult {
    let emit = node.borrow_by_key("emit");
    let emit = emit.get_string().unwrap();
    assert!(emit == "lvalue");

    let children = node.borrow_by_key("children").to_list();

    let mut ops: Vec<ImlOp> = Vec::new();

    for (i, item) in children.iter().enumerate() {
        let item = item.borrow();
        let item = item.get_dict().unwrap();

        let emit = item.borrow_by_key("emit");
        let emit = emit.get_string().unwrap();

        let store = if i < children.len() - 1 { false } else { store };

        match emit {
            capture if capture.starts_with("capture") => {
                let children = item.borrow_by_key("children");

                match capture {
                    "capture_expr" | "capture_alias" => {
                        ops.extend(
                            traverse_node_or_list(compiler, &children).into_ops(compiler, false),
                        );

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
                        let children = children.get_dict().unwrap();
                        let index = traverse_node_value(compiler, children).unwrap();

                        if store {
                            if hold {
                                ops.push(Op::StoreFastCaptureHold(index.to_addr()).into());
                            } else {
                                ops.push(Op::StoreFastCapture(index.to_addr()).into());
                            }
                        } else {
                            ops.push(Op::LoadFastCapture(index.to_addr()).into());
                        }
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
                ops.extend(
                    traverse_node_or_list(compiler, &item.borrow_by_key("children"))
                        .into_ops(compiler, true),
                );

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

    ImlResult::Ops(ops)
}

// Main traversal function, running recursively through the AST
fn traverse_node(compiler: &mut Compiler, node: &Dict) -> ImlResult {
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
            ops.push(Op::MakeAlias.into());

            ImlResult::Ops(ops)
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
                    "add" => ops.push(Op::InlineAdd.into()),
                    "sub" => ops.push(Op::InlineSub.into()),
                    "mul" => ops.push(Op::InlineMul.into()),
                    "div" => ops.push(Op::InlineDiv.into()),
                    _ => unreachable!(),
                }

                if *parts.last().unwrap() != "hold" {
                    ops.push(Op::Drop.into());
                }
            } else {
                ops.extend(traverse_node(compiler, value).into_ops(compiler, false));
                ops.extend(
                    traverse_node_lvalue(compiler, lvalue, true, *parts.last().unwrap() == "hold")
                        .into_ops(compiler, false),
                );
            }

            ImlResult::Ops(ops)
        }

        // attribute ------------------------------------------------------
        "attribute" => {
            let mut ops = traverse_node_or_list(compiler, &node.borrow_by_key("children"))
                .into_ops(compiler, true);

            insert_offset(&mut ops, node);
            ops.push(Op::LoadAttr.into());
            ImlResult::Ops(ops)
        }
        // begin ----------------------------------------------------------
        "begin" | "end" => {
            let ops = traverse_node_or_list(compiler, &node.borrow_by_key("children"))
                .into_ops(compiler, true);

            if let Scope::Parselet { begin, end, .. } = &mut compiler.scopes[0] {
                if emit == "begin" {
                    begin.push(ImlOp::from_vec(ops))
                } else {
                    end.push(ImlOp::from_vec(ops))
                }
            } else {
                compiler.errors.push(Error::new(
                    traverse_node_offset(node),
                    format!("'{}' may only be used in parselet scope", emit),
                ));
            }

            ImlResult::Empty
        }

        // block ----------------------------------------------------------
        "block" => {
            if let Some(children) = node.get("children") {
                compiler.push_block();
                let body =
                    traverse_node_or_list(compiler, &children.borrow()).into_ops(compiler, true);
                compiler.pop_block();

                ImlResult::Ops(if body.len() > 1 {
                    vec![ImlAlternation::new(body)]
                } else {
                    body
                })
            } else {
                ImlResult::Empty
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
                            ops.push(
                                Op::LoadStatic(compiler.define_value(Value::String(ident).into()))
                                    .into(),
                            );

                            nargs += 1;
                        }

                        other => unimplemented!("Unhandled parameter type {:?}", other),
                    }
                }
            }

            // When calling with nargs, create a nargs dict first
            if nargs > 0 {
                ops.push(Op::MakeDict(nargs).into());
            }

            // Push call position here
            insert_offset(&mut ops, node);

            // Perform static call or resolved rvalue call
            let callee = traverse_node_or_list(compiler, &children[0].borrow());

            if let ImlResult::Identifier(ident, offset) = callee {
                if utils::identifier_is_consumable(&ident) {
                    compiler.mark_consuming();
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
                    ops.push(Op::Call.into());
                } else if args > 0 && nargs == 0 {
                    ops.push(Op::CallArg(args).into());
                } else {
                    ops.push(Op::CallArgNamed(args).into())
                }
            }

            ImlResult::Ops(ops)
        }

        // capture --------------------------------------------------------
        "capture_alias" | "capture_expr" => {
            let children = node.borrow_by_key("children");

            let mut ops = traverse_node_or_list(compiler, &children).into_ops(compiler, false);
            ops.push(Op::LoadCapture.into());
            ImlResult::Ops(ops)
        }

        "capture_index" => {
            let children = node.borrow_by_key("children");

            let children = children.get_dict().unwrap();
            let index = traverse_node_value(compiler, children).unwrap();
            ImlResult::Ops(vec![Op::LoadFastCapture(index.to_addr()).into()])
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
                return ImlResult::Empty;
            }

            // Distinguish between pure values or an expression
            let value = value.get_dict().unwrap();

            // fixme: Restricted to pure values currently.
            let value = traverse_node_static(compiler, Some(&ident), value);

            if value.is_consuming() {
                if !utils::identifier_is_consumable(ident) {
                    compiler.errors.push(Error::new(
                        traverse_node_offset(node),
                        format!(
                            "Cannot assign constant '{}' as consumable. Use identifier starting in upper-case, e.g. '{}{}'",
                            ident, &ident[0..1].to_uppercase(), &ident[1..]
                        )
                    ));
                }
            } else if utils::identifier_is_consumable(ident) {
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
            compiler.resolve();

            ImlResult::Empty
        }

        // identifier -----------------------------------------------------
        "identifier" => {
            let name = node.borrow_by_key("value").to_string();

            // Check if identifier is valid
            if let Err(mut error) = identifier_is_valid(&name) {
                if let Some(offset) = traverse_node_offset(node) {
                    error.patch_offset(offset);
                }

                compiler.errors.push(error);
                ImlResult::Empty
            } else {
                ImlResult::Identifier(name, traverse_node_offset(node))
            }
        }

        // index ----------------------------------------------------------
        "index" => {
            let mut ops = traverse_node_or_list(compiler, &node.borrow_by_key("children"))
                .into_ops(compiler, true);

            insert_offset(&mut ops, node);
            ops.push(Op::LoadIndex.into()); // todo: in case value is an integer, use LoadFastIndex
            ImlResult::Ops(ops)
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
                    ops.push(
                        if parts[2] == "inc" {
                            Op::InlineInc
                        } else {
                            Op::InlineDec
                        }
                        .into(),
                    );
                }
                "post" => {
                    ops.extend(vec![
                        Op::Dup.into(),
                        Op::Rot2.into(),
                        if parts[2] == "inc" {
                            Op::InlineInc
                        } else {
                            Op::InlineDec
                        }
                        .into(),
                        Op::Drop.into(),
                    ]);
                }
                _ => unreachable!(),
            }

            ImlResult::Ops(ops)
        }

        // main -----------------------------------------------------------
        "main" => {
            let children = node.borrow_by_key("children");

            // When interactive and there's a scope, don't push, as the main scope
            // is kept to hold globals.
            if compiler.scopes.len() != 1 || !compiler.interactive {
                compiler.push_parselet(); // Main
            }

            let body = traverse_node_or_list(compiler, &children).into_ops(compiler, true);

            let main = compiler.pop_parselet(
                Some("__main__".to_string()),
                Vec::new(),
                match body.len() {
                    0 => Op::Nop.into(),
                    1 => body.into_iter().next().unwrap(),
                    _ => ImlAlternation::new(body),
                },
            );

            compiler.define_value(main.into());
            ImlResult::Empty
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
                        ops.extend(
                            traverse_node(compiler, &value.get_dict().unwrap())
                                .into_ops(compiler, true),
                        );

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
                        if let Ok(value) = match parts[2] {
                            "add" => left.add(&right),
                            "sub" => left.sub(&right),
                            "mul" => left.mul(&right),
                            "div" => left.div(&right),
                            _ => {
                                unimplemented!("op_binary_{}", parts[2]);
                            }
                        } {
                            return ImlResult::Value(value.into());
                        }
                    }

                    // Push operation position here
                    insert_offset(&mut ops, node);

                    // Otherwise, generate operational code
                    ops.extend(left.into_ops(compiler, true));
                    ops.extend(right.into_ops(compiler, true));

                    match parts[2] {
                        "add" => Op::Add.into(),
                        "sub" => Op::Sub.into(),
                        "mul" => Op::Mul.into(),
                        "div" => Op::Div.into(),
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
                        if let Ok(value) = match parts[2] {
                            "not" => value.not(),
                            "neg" => value.neg(),
                            _ => {
                                unimplemented!("op_unary_{}", parts[2]);
                            }
                        } {
                            return ImlResult::Value(value.into());
                        }
                    }

                    // Push operation position here
                    insert_offset(&mut ops, node);

                    ops.extend(res.into_ops(compiler, true));

                    match parts[2] {
                        "not" => Op::Not.into(),
                        "neg" => Op::Neg.into(),
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
                        return ImlResult::Value(
                            if match parts[2] {
                                "equal" => left == right,
                                "unequal" => left != right,
                                "lowerequal" => left <= right,
                                "greaterequal" => left >= right,
                                "lower" => left < right,
                                "greater" => left > right,
                                "and" => left.is_true() && right.is_true(),
                                "or" => left.is_true() || right.is_true(),
                                _ => {
                                    unimplemented!("op_compare_{}", parts[2]);
                                }
                            } {
                                Value::True.into()
                            } else {
                                Value::False.into()
                            },
                        );
                    }

                    // Otherwise, generate operational code
                    ops.extend(left.into_ops(compiler, false));

                    match parts[2] {
                        "and" => ImlIf::new_if_true(
                            ImlOp::from_vec(right.into_ops(compiler, false)),
                            Op::PushVoid.into(),
                        ),
                        "or" => ImlIf::new_if_false(
                            ImlOp::from_vec(right.into_ops(compiler, false)),
                            Op::PushVoid.into(),
                        ),
                        _ => {
                            ops.extend(right.into_ops(compiler, false));
                            match parts[2] {
                                "equal" => Op::Equal.into(),
                                "unequal" => Op::NotEqual.into(),
                                "lowerequal" => Op::LowerEqual.into(),
                                "greaterequal" => Op::GreaterEqual.into(),
                                "lower" => Op::Lower.into(),
                                "greater" => Op::Greater.into(),
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
                    if let ImlResult::Value(value) = &res {
                        if !value.is_consuming() {
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

                        if let ImlValue::Value(Value::Token(token)) = value {
                            if let Token::Char(ccl) = *token.clone() {
                                match parts[2] {
                                    // mod_pos on Token::Char becomes Token::Chars
                                    "pos" | "kle" => {
                                        let chars =
                                            ImlResult::Value(Token::Chars(ccl).into_value().into());

                                        if parts[2] == "pos" {
                                            return chars;
                                        }

                                        // mod_kle on Token::Char becomes Token::Chars.into_optional()
                                        return ImlResult::Ops(vec![ImlOp::from_vec(
                                            chars.into_ops(compiler, true),
                                        )
                                        .into_optional()]);
                                    }

                                    // mod_not on Token::Char becomes negated Token::Char
                                    "not" => {
                                        return ImlResult::Value(
                                            Token::Char(ccl.negate()).into_value().into(),
                                        );
                                    }
                                    _ => {}
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

                    let op = ImlOp::from_vec(res.into_ops(compiler, true));

                    match parts[2] {
                        "pos" => op.into_positive(),
                        "kle" => op.into_kleene(),
                        "opt" => op.into_optional(),
                        "peek" => ImlPeek::new(op),
                        "expect" => ImlExpect::new(op, Some("#todo".to_string())),  // todo!
                        "not" => ImlNot::new(op),
                        _ => unreachable!(),
                    }
                }

                "if" => {
                    let children = node.borrow_by_key("children");
                    let children = children.get_list().unwrap();

                    let condition = traverse_node_or_list(compiler, &children[0].borrow());
                    let then = traverse_node_or_list(compiler, &children[1].borrow());
                    let else_ = if children.len() == 3 {
                        Some(traverse_node_or_list(compiler, &children[2].borrow()))
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

                        return ImlResult::Value(Value::Void.into());
                    }

                    ops.extend(condition.into_ops(compiler, false));

                    ImlIf::new(
                        ImlOp::from_vec(then.into_ops(compiler, true)),
                        if let Some(else_) = else_ {
                            ImlOp::from_vec(else_.into_ops(compiler, true))
                        } else {
                            Op::PushVoid.into()
                        },
                    )
                }

                "for" => {
                    let children = node.borrow_by_key("children");
                    let children = children.get_list().unwrap();

                    // Initial
                    let initial = traverse_node_or_list(compiler, &children[0].borrow())
                        .into_ops(compiler, true);

                    compiler.push_loop();

                    let condition = traverse_node_or_list(compiler, &children[1].borrow())
                        .into_ops(compiler, true);
                    let mut body = traverse_node_or_list(compiler, &children[3].borrow())
                        .into_ops(compiler, true);
                    body.extend(
                        traverse_node_or_list(compiler, &children[2].borrow())
                            .into_ops(compiler, true),
                    );

                    compiler.pop_loop();

                    ImlLoop::new(
                        ImlOp::from_vec(initial),
                        ImlOp::from_vec(condition),
                        ImlOp::from_vec(body),
                    )
                }

                "loop" => {
                    let children = node.borrow_by_key("children");
                    let children = children.to_list();

                    compiler.push_loop();

                    let ret = match children.len() {
                        1 => ImlLoop::new(
                            ImlOp::Nop,
                            ImlOp::Nop,
                            ImlOp::from_vec(
                                traverse_node_or_list(compiler, &children[0].borrow())
                                    .into_ops(compiler, true),
                            ),
                        ),
                        2 => ImlLoop::new(
                            ImlOp::Nop,
                            ImlOp::from_vec(
                                traverse_node_or_list(compiler, &children[0].borrow())
                                    .into_ops(compiler, true),
                            ),
                            ImlOp::from_vec(
                                traverse_node_or_list(compiler, &children[1].borrow())
                                    .into_ops(compiler, true),
                            ),
                        ),
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

            ImlResult::Ops(ops)
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
            ImlResult::Ops(ops)
        }

        // sequence  ------------------------------------------------------
        "sequence" => {
            let children = node.borrow_by_key("children");
            let children = children.to_list();

            let mut ops = Vec::new();

            for node in &children {
                ops.extend(traverse_node_or_list(compiler, &node.borrow()).into_ops(compiler, true))
            }

            if ops.len() == 1 {
                ImlResult::Ops(ops)
            } else if ops.len() > 0 {
                ImlResult::Ops(vec![ImlSequence::new(ops)])
            } else {
                ImlResult::Empty
            }
        }

        // value ---------------------------------------------------------
        value if value.starts_with("value_") => {
            ImlResult::Value(traverse_node_value(compiler, node).into())
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

#[distributed_slice(BUILTINS)]
static AST: Builtin = Builtin {
    name: "ast",
    signature: "emit ? value",
    func: |context, args| {
        let emit = args[0].as_ref().unwrap();

        let mut ret = Dict::new();
        ret.insert("emit".to_string(), emit.clone());

        let value = match &args[1] {
            Some(value) => Some(value.clone()),
            None => context
                .collect(context.capture_start, false, true, false, 0)
                .unwrap_or(None),
        };

        if let Some(value) = value {
            // List or Dict values are classified as child nodes
            if value.borrow().get_list().is_some() || value.borrow().get_dict().is_some() {
                ret.insert("children".to_string(), value.clone());
            } else {
                ret.insert("value".to_string(), value.clone());
            }
        }

        // Store positions of reader start
        ret.insert(
            "offset".to_string(),
            Value::Addr(context.reader_start.offset).into(),
        );
        ret.insert(
            "row".to_string(),
            Value::Addr(context.reader_start.row as usize).into(),
        );
        ret.insert(
            "col".to_string(),
            Value::Addr(context.reader_start.col as usize).into(),
        );

        // Store positions of reader stop
        let current = context.runtime.reader.tell();

        ret.insert(
            "stop_offset".to_string(),
            Value::Addr(current.offset).into(),
        );
        ret.insert(
            "stop_row".to_string(),
            Value::Addr(current.row as usize).into(),
        );
        ret.insert(
            "stop_col".to_string(),
            Value::Addr(current.col as usize).into(),
        );

        Ok(Accept::Push(Capture::Value(
            Value::Dict(Box::new(ret)).into(),
            None,
            10,
        )))
    },
};

#[distributed_slice(BUILTINS)]
static AST_PRINT: Builtin = Builtin {
    name: "ast_print",
    signature: "ast",
    func: |_, args| {
        print(&args[0].as_ref().unwrap().borrow());
        Ok(Accept::Push(Capture::Value(Value::Void.into(), None, 10)))
    },
};
