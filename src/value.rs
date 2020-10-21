use std::rc::Rc;
use std::cell::{Ref, RefMut, RefCell};
use crate::map::Map;

//pub type RefValue = Rc<RefCell<Value>>;
pub type Complex = Map<String, RefValue>;

#[derive(Eq, PartialEq, Hash, Clone)]
pub enum Value {
    Unset,                  // unse
    Void,                   // void
    True,                   // true
    False,                  // false
    Integer(i64),           // integers
    //Float(f64),           // todo: Implement a hashable Float (i32, i32) or so...
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

    /* todo: Implement standard traits for this ... */

    // Get Value's boolean representation.
    pub fn to_bool(&self) -> Option<bool> {
        match self {
            Self::Unset | Self::Void => Some(false),
            Self::True => Some(true),
            Self::False => Some(false),
            Self::Integer(i) => Some(*i != 0),
            //Self::Float(f) => *f as i64,
            Self::String(s) => Some(s.len() != 0),
            Self::Complex(c) => Some(c.len() > 0),
            Self::Parselet(_) => Some(true)
        }
    }

    // Get Value's integer representation.
    pub fn to_integer(&self) -> Option<i64> {
        match self {
            Self::True => Some(1),
            Self::False => Some(0),
            Self::Integer(i) => Some(*i),
            //Self::Float(f) => *f as i64,
            Self::String(s) => {
                match s.parse::<i64>() {
                    Ok(i) => Some(i),
                    Err(_) => None
                }
            },
            _ => None
        }
    }

    // Get Value's float representation.
    pub fn to_float(&self) -> Option<f64> {
        match self {
            Self::True => Some(1.0),
            Self::False => Some(0.0),
            Self::Integer(i) => Some(*i as f64),
            //Self::Float(f) => *f
            Self::String(s) => {
                match s.parse::<f64>() {
                    Ok(f) => Some(f),
                    Err(_) => Some(0.0)
                }
            },
            _ => None
        }
    }

    // Get Value's string representation.
    pub fn to_string(&self) -> Option<String> {
        match self {
            Self::Unset => Some("unset".to_string()),
            Self::Void => Some("void".to_string()),
            Self::True => Some("true".to_string()),
            Self::False => Some("false".to_string()),
            Self::Integer(i) => Some(format!("{}", i)),
            //Self::Float(f) => format!("{}", f),
            Self::String(s) => Some(s.clone()),
            Self::Complex(c) => Some(format!("{:?}", c)),
            Self::Parselet(p) => Some(format!("{:?}", p))
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
