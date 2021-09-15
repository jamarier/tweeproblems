originales_yaml=$(shell ls *.yaml)
originales_twp=$(shell ls *.twp)
originales_tw=$(shell ls mathjax*.tw)
.PHONY: clean

../reveal/%.html: %.yaml ../target/debug/tweeproblems
	cargo run -- $< ../reveal  

#%.tw: %.yaml ../src/*.rs
#	cargo run -- $< ../reveal
#
#%.html: %.tw
#	tweego -o $@ $^

clean: 
	rm -f $(originales_yaml:yaml=tw)
	rm -f $(originales_yaml:yaml=html)
	rm -f $(originales_twp:twp=tw)
	rm -f $(originales_twp:twp=html)
	rm -f $(originales_tw:tw=html)

