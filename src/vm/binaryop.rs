// --- BinaryOp ----------------------------------------------------------------

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Div,
    DivI,
    Eq,
    Gt,
    GtEq,
    InlineAdd,
    InlineDiv,
    InlineDivI,
    InlineMod,
    InlineMul,
    InlineSub,
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
            Self::Add => "add",
            Self::Div => "div",
            Self::DivI => "divi",
            Self::Eq => "eq",
            Self::Gt => "gt",
            Self::GtEq => "gteq",
            Self::InlineAdd => "iadd",
            Self::InlineDiv => "idiv",
            Self::InlineDivI => "idivi",
            Self::InlineMod => "imod",
            Self::InlineMul => "imul",
            Self::InlineSub => "isub",
            Self::Lt => "lt",
            Self::LtEq => "lteq",
            Self::Mod => "mod",
            Self::Mul => "mul",
            Self::Neq => "neq",
            Self::Sub => "sub",
        }
    }
}
