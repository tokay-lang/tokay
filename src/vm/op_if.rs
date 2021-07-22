use super::*;

/** If-construct */

#[derive(Debug)]
pub struct If {
    then: Op,
    else_: Op,
}

impl If {
    pub fn new(then: Op, else_: Op) -> Op {
        Self { then, else_ }.into_op()
    }
}

impl Runable for If {
    fn run(&self, context: &mut Context) -> Result<Accept, Reject> {
        if context.pop().borrow().is_true() {
            self.then.run(context)
        } else {
            match &self.else_ {
                Op::Nop => Ok(Accept::Next),
                other => other.run(context),
            }
        }
    }

    fn resolve(&mut self, usages: &mut Vec<Vec<Op>>) {
        self.then.resolve(usages);
        self.else_.resolve(usages);
    }

    fn finalize(
        &mut self,
        statics: &Vec<RefValue>,
        stack: &mut Vec<(usize, bool)>,
    ) -> Option<(bool, bool)> {
        let then = self.then.finalize(statics, stack);

        if let Some((else_leftrec, else_nullable)) = self.else_.finalize(statics, stack) {
            if let Some((then_leftrec, then_nullable)) = then {
                Some((then_leftrec || else_leftrec, then_nullable || else_nullable))
            } else {
                Some((else_leftrec, else_nullable))
            }
        } else {
            then
        }
    }
}

impl std::fmt::Display for If {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "if {} else {}", self.then, self.else_)
    }
}
