GV := $(wildcard docs/drafts/*.gv)
SVG := $(patsubst %.gv,%.svg,$(GV))
PNG := $(patsubst %.gv,%.png,$(GV))

.PHONY: svg png
svg: $(SVG)
png: $(PNG)

%.svg: %.gv
		dot -Tsvg -o $@ $<
%.png: %.gv
		dot -Tpng -o $@ $<

.PHONY: clean
clean:
		@rm -f $(SVG)
