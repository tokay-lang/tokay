use super::*;

/** Peeking parser.

This parser runs its sub-parser and returns its result, but resets the reading-context
afterwards. It can be used to look ahead parsing constructs, but leaving the rest of the
parser back to its original position, to decide.

Due to Tokays memoizing features, the parsing will only be done once, and is remembered.
*/

#[derive(Debug)]
pub struct Peek {
    body: Op,
}

impl Peek {
    pub fn new(body: Op) -> Op {
        Self { body }.into_op()
    }
}

impl Runable for Peek {
    fn run(&self, context: &mut Context) -> Result<Accept, Reject> {
        let reader_start = context.runtime.reader.tell();
        let ret = self.body.run(context);
        context.runtime.reader.reset(reader_start);
        ret
    }

    fn finalize(
        &mut self,
        usages: &mut Vec<Vec<Op>>,
        statics: &Vec<RefValue>,
        leftrec: Option<&mut bool>,
        nullable: &mut bool,
        consumes: &mut bool,
    ) {
        self.body.replace_usage(usages);
        self.body
            .finalize(usages, statics, leftrec, nullable, consumes);
    }
}

impl std::fmt::Display for Peek {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "peek {}", self.body)
    }
}
