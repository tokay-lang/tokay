use std::rc::Rc;
use std::cell::{Ref, RefMut, RefCell};
use crate::map::Map;

//pub type RefValue = Rc<RefCell<Value>>;
pub type Complex = Map<String, RefValue>;

#[derive(PartialEq, Clone)]
pub enum Value {
    Unset,                  // unse
    Void,                   // void
    True,                   // true
    False,                  // false
    Integer(i64),           // integers
    Float(f64),             // float
    Addr(usize),            // usize
    String(String),         // string
    Complex(Box<Complex>),  // combined map/array type
    Parselet(usize)         // executable code parselet
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Unset => write!(f, "unset"),
            Value::Void => write!(f, "void"),
            Value::True => write!(f, "true"),
            Value::False => write!(f, "false"),
            Value::Integer(i) => write!(f, "{}", i),
            Value::Float(v) => write!(f, "{}", v),
            Value::Addr(a) => write!(f, "{}", a),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Complex(c) => {
                write!(f, "(")?;
                for (i, (k, v)) in c.iter().enumerate() {
                    if k.is_some() {
                        write!(f, "{:?}: {:?}", k.unwrap(), v)?;
                    } else {
                        write!(f, "{:?}", v)?;
                    }

                    if i + 1 < c.len() {
                        write!(f, ", ")?;
                    }
                }
                write!(f, ")")?;

                Ok(())
            },
            Value::Parselet(p) => write!(f, "@{:?}", p)
        }
    }
}

impl Value {
    // Create a RefValue from a Value.
    pub fn into_ref(self) -> RefValue {
        RefValue::new(self)
    }

    // Get Value's boolean meaning.
    pub fn is_true(&self) -> bool {
        match self {
            Self::True => true,
            Self::Integer(i) => *i != 0,
            Self::Float(f) => *f != 0.0,
            Self::String(s) => s.len() != 0,
            Self::Complex(c) => c.len() > 0,
            Self::Parselet(_) | Self::Addr(_) => true,
            _ => false
        }
    }

    // Get Value's integer representation.
    pub fn to_integer(&self) -> i64 {
        match self {
            Self::True => 1,
            Self::Integer(i) => *i,
            Self::Float(f) => *f as i64,
            Self::String(s) => {
                match s.parse::<i64>() {
                    Ok(i) => i,
                    Err(_) => 0
                }
            },
            _ => 0
        }
    }

    // Get Value's float representation.
    pub fn to_float(&self) -> f64 {
        match self {
            Self::True => 1.0,
            Self::Integer(i) => *i as f64,
            Self::Float(f) => *f,
            Self::String(s) => {
                match s.parse::<f64>() {
                    Ok(f) => f,
                    Err(_) => 0.0
                }
            },
            _ => 0.0
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
                match s.parse::<usize>() {
                    Ok(i) => i,
                    Err(_) => 0
                }
            },
            _ => 0
        }
    }

    // Get Value's string representation.
    pub fn to_string(&self) -> String {
        match self {
            Self::Unset => "unset".to_string(),
            Self::Void => "void".to_string(),
            Self::True => "true".to_string(),
            Self::False => "false".to_string(),
            Self::Integer(i) => format!("{}", i),
            Self::Addr(a) => format!("{}", a),
            Self::Float(f) => format!("{}", f),
            Self::String(s) => s.clone(),
            Self::Complex(c) => format!("{:?}", c),
            Self::Parselet(p) => format!("{:?}", p)
        }
    }

    // Get String from a value
    pub fn get_string(&self) -> Option<&str> {
        if let Self::String(s) = self {
            Some(&s)
        }
        else {
            None
        }
    }

    // Get Complex from a value
    pub fn get_complex(&self) -> Option<&Complex> {
        if let Self::Complex(c) = self {
            Some(&c)
        }
        else {
            None
        }
    }
}

/*
impl std::ops::Add for Value
{
    type Output = Value;

    fn add(self, rhs: Self) -> Self
    {
        match (self, rhs)
        {
            // When one is String...
            (Self::String(a), b) => Self::String(a + &b.to_string()),
            (a, Self::String(b)) => Self::String(a.to_string() + &b),

            /*
            // When one is Float...
            (Self::Float(a), b) => Self::Float(a + b.to_float()),
            (a, Self::Float(b)) => Self::Float(a.to_float() + b),
            */

            // All is threatened as Integer
            (a, b) => Self::Integer(a.to_integer() + b.to_integer()),
        }
    }
}

impl std::ops::Sub for Value
{
    type Output = Value;

    fn sub(self, rhs: Self) -> Self
    {
        match (self, rhs)
        {
            /*
            // When one is Float...
            (Self::Float(a), b) => Self::Float(a - b.to_float()),
            (a, Self::Float(b)) => Self::Float(a.to_float() - b),
            */

            // All is threatened as Integer
            (a, b) => Self::Integer(a.to_integer() - b.to_integer()),
        }
    }
}

impl std::ops::Mul for Value
{
    type Output = Value;

    fn mul(self, rhs: Self) -> Self
    {
        match (self, rhs)
        {
            // When one is String and one is something else...
            (Self::String(s), n) | (n, Self::String(s)) => {
                let n = n.to_integer();

                //Todo: why not extend `s`?
                let mut r = String::new();

                for _ in 0..n {
                    r += &s;
                }

                Self::String(r)
            },

            /*
            // When one is Float...
            (Self::Float(a), b) => Self::Float(a * b.to_float()),
            (a, Self::Float(b)) => Self::Float(a.to_float() * b),
            */

            // All is threatened as Integer
            (a, b) => Self::Integer(a.to_integer() * b.to_integer()),
        }
    }
}

impl std::ops::Div for Value
{
    type Output = Value;

    fn div(self, rhs: Self) -> Self
    {
        match (self, rhs)
        {
            /*
            // When one is Float...
            (Self::Float(a), b) => Self::Float(a / b.to_float()),
            (a, Self::Float(b)) => Self::Float(a.to_float() / b),
            */

            // All is threatened as Integer
            (a, b) => Self::Integer(a.to_integer() / b.to_integer()),
            // todo: handle float as results?
        }
    }
}
*/

// This is bat country...

#[test]
fn test_literal() {
    assert_eq!(Value::Integer(1337).to_integer(), Some(1337));
    assert_eq!(Value::Integer(-1337).to_integer(), Some(-1337));
    //assert_eq!(Value::Float(13.37).to_float(), 13.37);
    //assert_eq!(Value::Float(-13.37).to_float(), -13.37);
    assert_eq!(Value::String("hello world".to_string()).to_string(), Some("hello world".to_string()));
    assert_eq!(Value::Void.to_string(), Some("void".to_string()));
    assert_eq!(Value::True.to_string(), Some("true".to_string()));
    assert_eq!(Value::False.to_string(), Some("false".to_string()));
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

/*
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
