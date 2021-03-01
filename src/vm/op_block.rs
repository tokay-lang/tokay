use super::*;
use crate::value::RefValue;

/** Block parser.

A block parser defines either an alternation of sequences or a grouped sequence
of VM instructions. The compiler has to guarantee for correct usage of the block
parser.

Block parsers support static program constructs being left-recursive, and extend
the generated parse tree automatically until no more input can be consumed.
*/

#[derive(Debug)]
pub struct Block {
    leftrec: bool,
    all_leftrec: bool,
    items: Vec<(Op, bool)>,
}

impl Block {
    pub fn new(items: Vec<Op>) -> Op {
        Self {
            items: items.into_iter().map(|item| (item, false)).collect(),
            all_leftrec: false,
            leftrec: false,
        }
        .into_op()
    }
}

impl Runable for Block {
    fn run(&self, context: &mut Context) -> Result<Accept, Reject> {
        // Internal Block run function
        fn run(block: &Block, context: &mut Context, leftrec: bool) -> Result<Accept, Reject> {
            let mut res = Ok(Accept::Next);
            let reader_start = context.runtime.reader.tell();

            for (item, item_leftrec) in &block.items {
                // Skip over branches which don't match leftrec configuration
                if *item_leftrec != leftrec {
                    continue;
                }

                res = item.run(context);

                // Generally break on anything which is not Next.
                if !matches!(&res, Ok(Accept::Next) | Err(Reject::Next)) {
                    // Push only accepts when input was consumed, otherwise the
                    // push value is just discarded, except for the last item
                    // being executed.
                    if let Ok(Accept::Push(_)) = res {
                        // No consuming, no breaking!
                        if context.runtime.reader.capture_from(&reader_start).len() == 0 {
                            continue;
                        }
                    }

                    break;
                }
            }

            res
        }

        // Create a unique block id from the Block's address
        let id = self as *const Block as usize;

        // Check for an existing memo-entry, and return it in case of a match
        if let Some((reader_end, result)) =
            context.runtime.memo.get(&(context.reader_start.offset, id))
        {
            context.runtime.reader.reset(*reader_end);
            return result.clone();
        }

        if self.leftrec {
            //println!("Leftrec {:?}", self);

            // Left-recursive blocks are called in a loop until no more input
            // is consumed.

            let mut reader_end = context.reader_start;
            let mut result = if self.all_leftrec {
                Ok(Accept::Next)
            } else {
                Err(Reject::Next)
            };

            // Insert a fake memo entry to avoid endless recursion

            /* info: removing this fake entry does not affect program run!
            This is because of the leftrec parameter to internal run(),
            which only accepts non-left-recursive calls on the first run.
            As an additional fuse, this fake memo entry should anyway be kept.
            */
            context.runtime.memo.insert(
                (context.reader_start.offset, id),
                (reader_end, result.clone()),
            );

            let mut loops = 0;

            loop {
                let res = run(self, context, self.all_leftrec || loops > 0);

                match res {
                    // Hard reject
                    Err(Reject::Main) | Err(Reject::Error(_)) => return res,

                    // Soft reject
                    Err(_) => {
                        if loops == 0 {
                            return res;
                        } else {
                            break;
                        }
                    }

                    _ => {}
                }

                let pos = context.runtime.reader.tell();

                // Stop also when no more input was consumed
                if pos.offset <= reader_end.offset {
                    break;
                }

                result = res;

                // Save intermediate result in memo table
                reader_end = pos;
                context.runtime.memo.insert(
                    (context.reader_start.offset, id),
                    (reader_end, result.clone()),
                );

                // Reset reader & stack
                context.runtime.reader.reset(context.reader_start);
                context.runtime.stack.truncate(context.stack_start);
                context
                    .runtime
                    .stack
                    .resize(context.capture_start + 1, Capture::Empty);

                loops += 1;
            }

            context.runtime.reader.reset(reader_end);
            result
        } else {
            // Non-left-recursive block can be called directly.
            run(self, context, false)
        }
    }

    fn finalize(
        &mut self,
        statics: &Vec<RefValue>,
        usages: &mut Vec<Vec<Op>>,
        leftrec: &mut bool,
        nullable: &mut bool,
    ) {
        *nullable = false;
        self.all_leftrec = true;

        for (item, item_leftrec) in self.items.iter_mut() {
            item.replace_usage(usages);

            *item_leftrec = false;
            let mut item_nullable = true;

            item.finalize(statics, usages, item_leftrec, &mut item_nullable);

            if item_nullable {
                *nullable = true;
            }

            if *item_leftrec {
                self.leftrec = true;
            } else {
                self.all_leftrec = false;
            }
        }

        *leftrec = self.leftrec;
    }
}

impl std::fmt::Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (item, _) in &self.items {
            write!(f, "{}\n", item)?;
        }

        Ok(())
    }
}
