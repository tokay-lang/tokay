use super::*;

/** Loop-construct */

#[derive(Debug)]
pub struct Loop {
    body: ImlOp,
}

impl Loop {
    pub fn new(body: ImlOp) -> ImlOp {
        Self { body }.into_op()
    }
}

impl Runable for Loop {
    fn resolve(&mut self, usages: &mut Vec<Vec<ImlOp>>) {
        self.body.resolve(usages);
    }

    fn finalize(
        &mut self,
        statics: &Vec<RefValue>,
        stack: &mut Vec<(usize, bool)>,
    ) -> Option<Consumable> {
        self.body.finalize(statics, stack)
    }
}

impl std::fmt::Display for Loop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "loop {}", self.body)
    }
}
