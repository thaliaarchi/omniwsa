include ../../base.mk

SOURCES = $(shell find . -name '*.py')
OUT = $(SOURCES:.py=.out)

.PHONY: all
all: $(OUT)

%.out: %.py
	$(PYTHON2) $< > $@

.PHONY: clean
clean:
	rm -f $(OUT)
