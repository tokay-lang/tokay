// --- UnaryOp -----------------------------------------------------------------

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum UnaryOp {
    Dec,
    Inc,
    Neg,
    Not,
}

impl UnaryOp {
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Dec => "idec",
            Self::Inc => "iinc",
            Self::Neg => "neg",
            Self::Not => "not",
        }
    }
}
