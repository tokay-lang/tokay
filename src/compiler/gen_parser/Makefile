.PHONY: .FORCE

../parser.rs: .FORCE
	awk -f etareneg.awk $@ >$@.1
	mv $@.1 $@
