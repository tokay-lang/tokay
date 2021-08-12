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
    ) -> Option<Consumable> {
        let then = self.then.finalize(statics, stack);

        if let Some(else_) = self.else_.finalize(statics, stack) {
            if let Some(then) = then {
                Some(Consumable {
                    leftrec: then.leftrec || else_.leftrec,
                    nullable: then.nullable || else_.nullable,
                })
            } else {
                Some(else_)
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
