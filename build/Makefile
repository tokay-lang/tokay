.PHONY: .FORCE
ETARENEG=cd ../src && awk -f ../build/etareneg.awk

all:
	@echo Please call one of the following targets:
	@echo ""
	@echo "  make parser         - build the parser"
	@echo "  make reset_parser   - reset parser from git"
	@echo "  make builtins       - build the builtin registry"
	@echo "  make reset_builtins - reset builtin registry from git"
	@echo ""

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
