.PHONY: .FORCE
ETARENEG=cd ../src && awk -f ../build/etareneg.awk

all:
	@echo No target specified. See README.md for details.
	@echo ""
	@echo "  make builtins    update src/_builtins.rs from src/"
	@echo "  make parser      update src/compiler/parse.rs from src/compiler/tokay.tok"
	@echo "  make prelude     update src/compiler/prelude.rs from src/prelude.tok"
	@echo ""

# builtins --------------------------------------------------------------------
BUILTINS=../src/_builtins.rs

builtins: $(BUILTINS)

$(BUILTINS): .FORCE
	$(ETARENEG) $@ >$@.1 && mv $@.1 $@

show-builtins:
	$(ETARENEG) $(BUILTINS) 2>/dev/null

reset-builtins:
	git checkout $(BUILTINS)


# parser ----------------------------------------------------------------------
PARSER=../src/compiler/parser.rs

parser: $(PARSER)

$(PARSER): .FORCE
	$(ETARENEG) $@ >$@.1 && mv $@.1 $@

show-parser:
	$(ETARENEG) $(PARSER) 2>/dev/null

reset-parser:
	git checkout $(PARSER)

# prelude ----------------------------------------------------------------------
PRELUDE=../src/compiler/prelude.rs

prelude: $(PRELUDE)

$(PRELUDE): .FORCE
	$(ETARENEG) $@ >$@.1 && mv $@.1 $@

show-prelude:
	$(ETARENEG) $(PRELUDE) 2>/dev/null

reset-prelude:
	git checkout $(PRELUDE)
