use std::cell::{Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::rc::Rc;

use crate::builtin;
use crate::vm::{Accept, Context, Parselet, Reject};

pub trait BorrowByKey {
    fn borrow_by_key(&self, key: &str) -> Ref<Value>;
    fn borrow_by_key_mut(&self, key: &str) -> RefMut<Value>;
}

pub trait BorrowByIdx {
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

// https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=4dae75b00e3fe033dc495b279b52684f

// --- RefValue ---------------------------------------------------------------

pub type RefValue = Rc<RefCell<Value>>;

// --- List -------------------------------------------------------------------
pub type List = Vec<RefValue>;

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

// --- Dict -------------------------------------------------------------------
pub type Dict = HashMap<String, RefValue>;

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

// --- Value ------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Void,  // void
    Null,  // null
    True,  // true
    False, // false

    Integer(i64),   // integers
    Float(f64),     // float
    Addr(usize),    // usize
    String(String), // string

    List(Box<List>), // list
    Dict(Box<Dict>), // dict

    Parselet(Rc<RefCell<Parselet>>), // tokay parselet
    Builtin(usize),                  // builtin parselet
}

#[macro_export]
macro_rules! value {
    ( [ $($key:literal => $value:tt),* ] ) => {
        {
            let mut dict = Dict::new();
            $( dict.insert($key.to_string(), value!($value)); )*
            Value::Dict(Box::new(dict)).into_ref()
        }
    };

    ( [ $($value:tt),* ] ) => {
        {
            let mut list = List::new();
            $( list.push(value!($value)); )*
            Value::List(Box::new(list)).into_ref()
        }
    };

    ( $value:expr ) => {
        if let Some(value) = (&$value as &std::any::Any).downcast_ref::<bool>()
        {
            if *value {
                Value::True.into_ref()
            }
            else {
                Value::False.into_ref()
            }
        }
        else
        if let Some(value) = (&$value as &std::any::Any).downcast_ref::<f64>()
        {
            Value::Float(*value).into_ref()
        }
        else
        if let Some(value) = (&$value as &std::any::Any).downcast_ref::<i32>()
        {
            Value::Integer(*value as i64).into_ref()
        }
        else
        if let Some(value) = (&$value as &std::any::Any).downcast_ref::<i64>()
        {
            Value::Integer(*value).into_ref()
        }
        else {
            Value::String($value.to_string()).into_ref()
        }
    }
}

/*
impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Void => write!(f, "void"),
            Value::Null => write!(f, "null"),
            Value::True => write!(f, "true"),
            Value::False => write!(f, "false"),
            Value::Integer(i) => write!(f, "{}", i),
            Value::Float(v) => write!(f, "{}", v),
            Value::Addr(a) => write!(f, "{}", a),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::List(l) => {
                write!(f, "(")?;

                for (i, v) in l.iter().enumerate() {
                    write!(f, "{:?}", v)?;

                    if i + 1 < l.len() {
                        write!(f, ", ")?;
                    }
                }

                write!(f, ")")
            },
            Value::Dict(d) => {
                write!(f, "(")?;

                for (i, (k, v)) in d.iter().enumerate() {
                    write!(f, "{:?}: {:?}", k, v)?;

                    if i + 1 < d.len() {
                        write!(f, ", ")?;
                    }
                }

                write!(f, ")")
            },
            Value::Parselet(p) => write!(f, "@{:?}", p)
        }
    }
}
*/

impl Value {
    // Create a RefValue from a Value.
    pub fn into_ref(self) -> RefValue {
        RefValue::new(RefCell::new(self))
    }

    // Convert a RefValue into a Value
    pub fn from_ref(this: RefValue) -> Result<Value, RefValue> {
        match Rc::try_unwrap(this) {
            Ok(this) => Ok(this.into_inner()),
            Err(this) => Err(this),
        }
    }

    // Get Value's boolean meaning.
    pub fn is_true(&self) -> bool {
        match self {
            Self::True => true,
            Self::Integer(i) => *i != 0,
            Self::Float(f) => *f != 0.0,
            Self::String(s) => s.len() != 0,
            Self::List(l) => l.len() > 0,
            Self::Dict(d) => d.len() > 0,
            Self::Builtin(_) | Self::Parselet(_) | Self::Addr(_) => true,
            _ => false,
        }
    }

    // Get Value's integer representation.
    pub fn to_integer(&self) -> i64 {
        match self {
            Self::True => 1,
            Self::Integer(i) => *i,
            Self::Float(f) => *f as i64,
            Self::String(s) => {
                // todo: parseInt-like behavior?
                match s.parse::<i64>() {
                    Ok(i) => i,
                    Err(_) => 0,
                }
            }
            _ => 0,
        }
    }

    // Get Value's float representation.
    pub fn to_float(&self) -> f64 {
        match self {
            Self::True => 1.0,
            Self::Integer(i) => *i as f64,
            Self::Float(f) => *f,
            Self::String(s) => {
                // todo: parseFloat-like behavior?
                match s.parse::<f64>() {
                    Ok(f) => f,
                    Err(_) => 0.0,
                }
            }
            _ => 0.0,
        }
    }

    // Get Value's integer representation.
    pub fn to_addr(&self) -> usize {
        match self {
            Self::True => 1,
            Self::Integer(i) => *i as usize,
            Self::Float(f) => *f as usize,
            Self::Addr(a) => *a,
            Self::String(s) => {
                // todo: parseInt-like behavior?
                match s.parse::<usize>() {
                    Ok(i) => i,
                    Err(_) => 0,
                }
            }
            _ => 0,
        }
    }

    // Get Value's string representation.
    pub fn to_string(&self) -> String {
        match self {
            Self::Void => "".to_string(),
            Self::Null => "null".to_string(),
            Self::True => "true".to_string(),
            Self::False => "false".to_string(),
            Self::Integer(i) => format!("{}", i),
            Self::Addr(a) => format!("{}", a),
            Self::Float(f) => format!("{}", f),
            Self::String(s) => s.clone(),
            Self::List(l) => format!("{:?}", l),
            Self::Dict(d) => format!("{:?}", d),
            Self::Parselet(p) => format!("{:?}", p),
            Self::Builtin(b) => format!("{:?}", b),
        }
    }

    // Get a Value's list representation.
    pub fn to_list(&self) -> List {
        if let Self::List(l) = self {
            *l.clone()
        } else {
            let mut l = List::new();
            l.push(self.clone().into_ref());
            l
        }
    }

    // Get a Value's dict representation.
    pub fn to_dict(&self) -> Dict {
        if let Self::Dict(d) = self {
            *d.clone()
        } else {
            let mut d = Dict::new();
            d.insert("0".to_string(), self.clone().into_ref());
            d
        }
    }

    // Extract String from a value
    pub fn get_string(&self) -> Option<&str> {
        if let Self::String(s) = self {
            Some(&s)
        } else {
            None
        }
    }

    // Extract List from a value
    pub fn get_list(&self) -> Option<&List> {
        if let Self::List(l) = self {
            Some(&l)
        } else {
            None
        }
    }

    // Extract Dict from a value
    pub fn get_dict(&self) -> Option<&Dict> {
        if let Self::Dict(d) = self {
            Some(&d)
        } else {
            None
        }
    }

    // Check whether a value is callable
    pub fn is_callable(&self) -> bool {
        matches!(self, Value::Parselet(_) | Value::Builtin(_))
    }

    // Call
    pub fn call(&self, context: &mut Context) -> Result<Accept, Reject> {
        match self {
            Value::Builtin(addr) => builtin::call(*addr, context),
            Value::Parselet(parselet) => parselet.borrow().run(context.runtime, false),

            _ => {
                panic!("{:?} cannot be called", self)
            }
        }
    }
}

impl<'a, 'b> std::ops::Add<&'b Value> for &'a Value {
    type Output = Value;

    fn add(self, rhs: &'b Value) -> Value {
        match (self, rhs) {
            // When one is String...
            (Value::String(a), b) => Value::String(a.to_owned() + &b.to_string()),
            (a, Value::String(b)) => Value::String(a.to_string() + &b),

            // When one is Float...
            (Value::Float(a), b) => Value::Float(a + b.to_float()),
            (a, Value::Float(b)) => Value::Float(a.to_float() + b),

            // All is threatened as Integer
            (a, b) => Value::Integer(a.to_integer() + b.to_integer()),
        }
    }
}

impl<'a, 'b> std::ops::Sub<&'b Value> for &'a Value {
    type Output = Value;

    fn sub(self, rhs: &'b Value) -> Value {
        match (self, rhs) {
            // When one is Float...
            (Value::Float(a), b) => Value::Float(a - b.to_float()),
            (a, Value::Float(b)) => Value::Float(a.to_float() - b),

            // All is threatened as Integer
            (a, b) => Value::Integer(a.to_integer() - b.to_integer()),
        }
    }
}

impl<'a, 'b> std::ops::Mul<&'b Value> for &'a Value {
    type Output = Value;

    fn mul(self, rhs: &'b Value) -> Value {
        match (self, rhs) {
            // When one is String and one is something else...
            (Value::String(s), n) | (n, Value::String(s)) => {
                let n = n.to_integer();

                //Todo: why not extend `s`?
                let mut r = String::new();

                for _ in 0..n {
                    r += &s;
                }

                Value::String(r)
            }

            // When one is Float...
            (Value::Float(a), b) => Value::Float(a * b.to_float()),
            (a, Value::Float(b)) => Value::Float(a.to_float() * b),

            // All is threatened as Integer
            (a, b) => Value::Integer(a.to_integer() * b.to_integer()),
        }
    }
}

impl<'a, 'b> std::ops::Div<&'b Value> for &'a Value {
    type Output = Value;

    fn div(self, rhs: &'b Value) -> Value {
        match (self, rhs) {
            // When one is Float...
            (Value::Float(a), b) => Value::Float(a / b.to_float()),
            (a, Value::Float(b)) => Value::Float(a.to_float() / b),

            // All is threatened as Integer
            (a, b) => Value::Integer(a.to_integer() / b.to_integer()),
            // todo: handle float as results?
        }
    }
}

// This is bat country...
/*
#[test]
fn test_literal() {
    assert_eq!(Value::Integer(1337).to_integer().unwrap(), Some(1337));
    assert_eq!(Value::Integer(-1337).to_integer().unwrap(), Some(-1337));
    //assert_eq!(Value::Float(13.37).to_float(), 13.37);
    //assert_eq!(Value::Float(-13.37).to_float(), -13.37);
    assert_eq!(Value::String("hello world".to_string()).to_string().unwrap(), Some("hello world".to_string()));
    assert_eq!(Value::Void.to_string().unwrap(), Some("void".to_string()));
    assert_eq!(Value::True.to_string().unwrap(), Some("true".to_string()));
    assert_eq!(Value::False.to_string().unwrap(), Some("false".to_string()));
}

#[test]
fn test_conversion() {
    let v = Value::Void;
    assert_eq!(v.to_string(), Some("void".to_string()));
    assert_eq!(v.to_integer(), None);
    assert_eq!(v.to_bool(), Some(false));

    /*
    let f = Value::Float(1.337);
    assert_eq!(f.to_float(), 1.337);
    assert_eq!(f.to_string(), "1.337");
    assert_eq!(f.to_float(), 1.337);
    assert_eq!(f.to_integer(), 1);
    */

    let s = Value::String("42".to_string());
    assert_eq!(s.to_string(), Some("42".to_string()));
    assert_eq!(s.to_integer(), Some(42));
    assert_eq!(s.to_float(), Some(42.0));
    assert_eq!(s.to_string(), Some("42".to_string()));

    let s = Value::String("gunshot42".to_string());
    assert_eq!(s.to_string(), Some("gunshot42".to_string()));
    assert_eq!(s.to_integer(), Some(0));
}

#[test]
fn test_add() {
    let x = Value::Integer(42);
    let y = Value::String("hello".to_string());
    assert_eq!(x + y, Value::String("42hello".to_string()));

    let x = Value::Integer(42);
    let y = Value::String("hello".to_string());
    assert_eq!(y + x, Value::String("hello42".to_string()));

    let x = Value::Integer(42);
    let y = Value::Integer(23);
    assert_eq!(x + y, Value::Integer(65));

    //let x = Value::Integer(42);
    //let y = Value::Float(1.337);
    //assert_eq!(x + y, Value::Float(43.337));
}


#[test]
fn test_mul() {
    let x = Value::Integer(42);
    let y = Value::Integer(23);
    assert_eq!(x * y, Value::Integer(966));

    let x = Value::Integer(3);
    let y = Value::String("hello".to_string());
    assert_eq!(x * y, Value::String("hellohellohello".to_string()));

    assert_eq!(Value::Bool(true) * Value::String("hello".to_string()), Value::String("hello".to_string()));
    assert_eq!(Value::Bool(false) * Value::String("hello".to_string()), Value::String("".to_string()));

    //let x = Value::Integer(42);
    //let y = Value::Float(1.337);
    //assert_eq!(x + y, Value::Float(43.337));
}

*/

/*
#[derive(Clone, PartialEq)]
pub struct RefValue(Rc<RefCell<Value>>);

impl RefValue {
    pub fn new(value: Value) -> Self {
        Self(Rc::new(RefCell::new(value)))
    }

    pub fn borrow(&self) -> Ref<Value> {
        self.0.borrow()
    }

    pub fn borrow_mut(&self) -> RefMut<Value> {
        self.0.borrow_mut()
    }

    pub fn into_value(this: RefValue) -> Result<Value, RefValue> {
        match Rc::try_unwrap(this.0) {
            Ok(this) => Ok(this.into_inner()),
            Err(this) => Err(RefValue(this))
        }
    }
}

impl std::fmt::Debug for RefValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.borrow())
    }
}
*/
