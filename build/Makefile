.PHONY: .FORCE
ETARENEG=cd ../src && awk -f ../build/etareneg.awk

all:
	@echo No target specified. See README.md for details.
	@echo ""
	@echo "  make builtins    update src/_builtins.rs from src/"
	@echo "  make parser      update src/compiler/parse.rs from src/compiler/tokay.tok"
	@echo ""

# parser ----------------------------------------------------------------------
PARSER=../src/compiler/parser.rs

parser: $(PARSER)

$(PARSER): .FORCE
	$(ETARENEG) $@ >$@.1 && mv $@.1 $@

show_parser:
	$(ETARENEG) $(PARSER)

reset_parser:
	git checkout $(PARSER)

# builtin ---------------------------------------------------------------------
BUILTINS=../src/_builtins.rs

builtins: $(BUILTINS)

$(BUILTINS): .FORCE
	$(ETARENEG) $@ >$@.1 && mv $@.1 $@

show_builtins:
	$(ETARENEG) $(BUILTINS)

reset_builtins:
	git checkout $(BUILTINS)
