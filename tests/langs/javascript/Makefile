include ../../base.mk

SOURCES = $(shell find . -name '*.js')
OUT = $(SOURCES:.js=.out) $(patsubst %, toLocaleLowerCase_to_ascii.%.out, $(LOCALES))
LOCALES = en-US tr az

.PHONY: all
all: $(OUT)

%.out: %.js
	@printf 'Node.js ' > $@
	node --version >> $@
	$(NODE) $< >> $@

toLocaleLowerCase_to_ascii.%.out: toLocaleLowerCase_to_ascii.js
	@printf 'Node.js ' > $@
	node --version >> $@
	LANG=$* $(NODE) $< >> $@

.PHONY: clean
clean:
	rm -f $(OUT)
