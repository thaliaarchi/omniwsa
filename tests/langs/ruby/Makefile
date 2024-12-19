include ../../base.mk

SOURCES = $(shell find . -name '*.rb')
OUT = $(SOURCES:.rb=.out)

.PHONY: all
all: $(OUT)

%.out: %.rb
	$(RUBY) $< > $@

.PHONY: clean
clean:
	rm -f $(OUT)
