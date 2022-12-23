../src/compiler/grammar.rs: Makefile ../examples/tokay.tok
	awk -f etareneg.awk $@ >$@.1
	mv $@.1 $@

touch:
	touch ../examples/tokay.tok
