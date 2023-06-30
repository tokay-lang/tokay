.PHONY: .FORCE
ETARENEG=cd ../src && awk -f ../build/etareneg.awk

all:
	@echo No target specified. See README.md for details.

# parser ----------------------------------------------------------------------
PARSER=../src/compiler/parser.rs

parser: $(PARSER)

$(PARSER): .FORCE
	$(ETARENEG) $@ >$@.1 && mv $@.1 $@

reset_parser:
	git checkout $(PARSER)

# builtin ---------------------------------------------------------------------
BUILTINS=../src/_builtins.rs

builtins: $(BUILTINS)

$(BUILTINS): .FORCE
	$(ETARENEG) $@ >$@.1 && mv $@.1 $@

reset_builtin:
	git checkout $(BUILTINS)
