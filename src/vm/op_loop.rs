use super::*;

/** If-construct */

#[derive(Debug)]
pub struct Loop {
    body: Op,
}

impl Loop {
    pub fn new(body: Op) -> Op {
        Self { body }.into_op()
    }
}

impl Runable for Loop {
    fn run(&self, context: &mut Context) -> Result<Accept, Reject> {
        let capture_start = context.runtime.stack.len();

        loop {
            let ret = self.body.run(context);
            //println!("loop {:?}", ret);
            match ret {
                Ok(Accept::Next | Accept::Continue) => {
                    context.runtime.stack.truncate(capture_start);
                }
                Ok(Accept::Break(Some(value))) => {
                    break Ok(Accept::Push(Capture::Value(value, None, 10)))
                }
                Ok(Accept::Break(None)) => break Ok(Accept::Next),
                other => break other,
            }
        }
    }

    fn resolve(&mut self, usages: &mut Vec<Vec<Op>>) {
        self.body.resolve(usages);
    }

    fn finalize(
        &mut self,
        statics: &Vec<RefValue>,
        stack: &mut Vec<(usize, bool)>,
    ) -> Option<(bool, bool)> {
        self.body.finalize(statics, stack)
    }
}

impl std::fmt::Display for Loop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "loop {}", self.body)
    }
}
