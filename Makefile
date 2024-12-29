.PHONY: .FORCE
ETARENEG=awk -f etareneg.awk

help:
	@echo "No target specified."
	@echo "See README.md build-section for details."
	@echo ""
	@echo "  make builtins  update src/_builtins.rs"
	@echo "  make parser    update src/compiler/parser.rs from src/compiler/tokay.tok"
	@echo "  make prelude   update src/compiler/prelude.rs from src/prelude.tok"
	@echo ""
	@echo "This is the Tokay source generation toolchain."
	@echo "To just build Tokay, simply use 'cargo build'."

all:
	make prelude
	make parser
	make builtins

# builtins --------------------------------------------------------------------
BUILTINS=src/_builtins.rs

builtins: $(BUILTINS)

$(BUILTINS): .FORCE
	$(ETARENEG) $@ >$@.1 && mv $@.1 $@

show-builtins:
	$(ETARENEG) $(BUILTINS) 2>/dev/null

reset-builtins:
	git checkout $(BUILTINS)


# parser ----------------------------------------------------------------------
PARSER=src/compiler/parser.rs

parser: $(PARSER)

$(PARSER): .FORCE
	$(ETARENEG) $@ >$@.1 && mv $@.1 $@

show-parser:
	$(ETARENEG) $(PARSER) 2>/dev/null

reset-parser:
	git checkout $(PARSER)

# prelude ----------------------------------------------------------------------
PRELUDE=src/compiler/prelude.rs

prelude: $(PRELUDE)

$(PRELUDE): .FORCE
	$(ETARENEG) $@ >$@.1 && mv $@.1 $@

show-prelude:
	$(ETARENEG) $(PRELUDE) 2>/dev/null

reset-prelude:
	git checkout $(PRELUDE)
