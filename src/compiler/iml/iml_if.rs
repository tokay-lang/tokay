use super::*;

/** If-construct */

#[derive(Debug)]
pub struct If {
    peek: bool,
    test: bool,
    then: ImlOp,
    else_: ImlOp,
}

impl If {
    pub fn new(then: ImlOp, else_: ImlOp) -> ImlOp {
        Self {
            peek: false,
            test: true,
            then,
            else_,
        }
        .into_op()
    }

    pub fn new_if_not(then: ImlOp, else_: ImlOp) -> ImlOp {
        Self {
            peek: false,
            test: false,
            then,
            else_,
        }
        .into_op()
    }

    pub fn new_if_true(then: ImlOp, else_: ImlOp) -> ImlOp {
        Self {
            peek: true,
            test: true,
            then,
            else_,
        }
        .into_op()
    }

    pub fn new_if_false(then: ImlOp, else_: ImlOp) -> ImlOp {
        Self {
            peek: true,
            test: false,
            then,
            else_,
        }
        .into_op()
    }
}

impl Runable for If {
    fn run(&self, context: &mut Context) -> Result<Accept, Reject> {
        if self.peek {
            if context.peek().borrow().is_true() == self.test {
                context.pop();
                self.then.run(context)
            } else {
                match &self.else_ {
                    ImlOp::Nop => Ok(Accept::Next),
                    other => other.run(context),
                }
            }
        } else if context.pop().borrow().is_true() == self.test {
            self.then.run(context)
        } else {
            match &self.else_ {
                ImlOp::Nop => Ok(Accept::Next),
                other => other.run(context),
            }
        }
    }

    fn resolve(&mut self, usages: &mut Vec<Vec<ImlOp>>) {
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
