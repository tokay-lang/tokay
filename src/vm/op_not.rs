use super::*;

/** Negate parser.

This parser runs its sub-parser and returns its negated result, so that an accept becomes
rejected and vice-versa.
*/

#[derive(Debug)]
pub struct Not {
    body: Op,
}

impl Not {
    pub fn new(body: Op) -> Op {
        Self { body }.into_op()
    }
}

impl Runable for Not {
    fn run(&self, context: &mut Context) -> Result<Accept, Reject> {
        let res = match self.body.run(context) {
            Ok(_) => Err(Reject::Next),
            Err(Reject::Error(err)) => Err(Reject::Error(err)),
            Err(_) => Ok(Accept::Next),
        };

        println!("res = {:?}", res);
        res
    }

    fn finalize(
        &mut self,
        usages: &mut Vec<Vec<Op>>,
        statics: &Vec<RefValue>,
        leftrec: &mut bool,
        nullable: &mut bool,
        consumes: &mut bool,
    ) {
        self.body.replace_usage(usages);
        self.body
            .finalize(usages, statics, leftrec, nullable, consumes);
    }
}

impl std::fmt::Display for Not {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not {}", self.body)
    }
}
