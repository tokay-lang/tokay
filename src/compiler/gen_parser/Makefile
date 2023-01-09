.PHONY: .FORCE

PARSER=../parser.rs

$(PARSER): .FORCE
	awk -f etareneg.awk $@ >$@.1
	mv $@.1 $@

reset:
	git checkout $(PARSER)
