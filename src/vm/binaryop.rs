// --- BinaryOp ----------------------------------------------------------------

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Div,
    DivI,
    Eq,
    Gt,
    GtEq,
    Inline(Box<BinaryOp>),
    Lt,
    LtEq,
    Mod,
    Mul,
    Neq,
    Sub,
}

impl BinaryOp {
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Inline(op) => match **op {
                Self::Add => "iadd",
                Self::Div => "idiv",
                Self::DivI => "idivi",
                Self::Mod => "imod",
                Self::Mul => "imul",
                Self::Sub => "isub",
                _ => unimplemented!(),
            },
            Self::Add => "add",
            Self::Div => "div",
            Self::DivI => "divi",
            Self::Eq => "eq",
            Self::Gt => "gt",
            Self::GtEq => "gteq",
            Self::Lt => "lt",
            Self::LtEq => "lteq",
            Self::Mod => "mod",
            Self::Mul => "mul",
            Self::Neq => "neq",
            Self::Sub => "sub",
        }
    }
}
