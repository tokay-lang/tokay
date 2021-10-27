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

    fn compile(&self, parselet: &ImlParselet) -> Vec<Op> {
        let mut ret = Vec::new();

        // Clone on peek
        if self.peek {
            ret.push(Op::Clone);
        }

        // Then-part
        let then = self.then.compile(parselet);

        let backpatch = ret.len();
        ret.push(Op::Nop);

        if self.peek {
            ret.push(Op::Drop)
        }

        let mut jump = then.len() + 1;
        ret.extend(then);

        if !self.peek {
            // Else-part
            let else_ = self.else_.compile(parselet);

            if !else_.is_empty() {
                ret.push(Op::Forward(else_.len() + 1));
                jump += 1;
                ret.extend(else_);
            }
        } else {
            jump += 1;
        }

        // Insert the final condition and its failure target.
        if self.test {
            ret[backpatch] = Op::ForwardIfFalse(jump);
        } else {
            ret[backpatch] = Op::ForwardIfTrue(jump);
        }

        ret
    }
}

impl std::fmt::Display for If {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "if {} else {}", self.then, self.else_)
    }
}
