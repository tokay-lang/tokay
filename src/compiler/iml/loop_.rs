use super::*;

/** Loop-construct */

#[derive(Debug)]
pub struct Loop {
    consuming: Option<Consumable>, // Consumable state
    init: ImlOp,
    condition: ImlOp,
    body: ImlOp,
}

impl Loop {
    pub fn new(init: ImlOp, condition: ImlOp, body: ImlOp) -> ImlOp {
        Self {
            consuming: None,
            init,
            condition,
            body,
        }
        .into_op()
    }
}

impl Compileable for Loop {
    fn resolve(&mut self, usages: &mut Vec<Vec<ImlOp>>) {
        self.init.resolve(usages);
        self.condition.resolve(usages);
        self.body.resolve(usages);
    }

    fn finalize(
        &mut self,
        values: &Vec<ImlValue>,
        stack: &mut Vec<(usize, bool)>,
    ) -> Option<Consumable> {
        let mut ret: Option<Consumable> = None;

        for part in [
            self.init.finalize(values, stack),
            self.condition.finalize(values, stack),
            self.body.finalize(values, stack),
        ] {
            if let Some(part) = part {
                ret = if let Some(ret) = ret {
                    Some(Consumable {
                        leftrec: ret.leftrec || part.leftrec,
                        nullable: ret.nullable || part.nullable,
                    })
                } else {
                    Some(part)
                }
            }
        }

        self.consuming = ret.clone();

        ret
    }

    fn compile(&self, parselet: &ImlParselet) -> Vec<Op> {
        let mut ret = Vec::new();

        ret.extend(self.init.compile(parselet));

        let mut body = self.condition.compile(parselet);
        if !body.is_empty() {
            body.push(Op::ForwardIfTrue(2));
            body.push(Op::Break);
        }

        body.extend(self.body.compile(parselet));

        ret.push(Op::Loop(
            body.len() + if self.consuming.is_some() { 3 } else { 2 },
        ));
        if self.consuming.is_some() {
            ret.push(Op::Fuse(body.len() + 2));
        }

        ret.extend(body);
        ret.push(Op::Continue);

        if self.consuming.is_some() {
            ret.push(Op::Break);
        }

        ret
    }
}

impl std::fmt::Display for Loop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "loop {}", self.body)
    }
}
