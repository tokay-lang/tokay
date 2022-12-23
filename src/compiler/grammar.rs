use crate::RefValue;
use crate::value;

pub(super) fn tokay() -> RefValue {
    /*GENERATE cargo run -- "`sed 's/ast("main")/ast2rust(ast("main"))/g' ../examples/tokay.tok`" -- ../examples/tokay.tok */
    value!(void)
    /*ETARENEG*/
}
