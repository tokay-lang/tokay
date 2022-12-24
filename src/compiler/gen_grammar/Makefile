.PHONY: .FORCE

../grammar.rs: .FORCE
	awk -f etareneg.awk $@ >$@.1
	mv $@.1 $@
