-include ../config.mk
-include ../config.tmpl.mk

WSA = $(shell find . -name '*.wsa')
WS = $(WSA:.wsa=.ws)

.PHONY: all
all: $(WS)

%.ws: %.wsa
	$(RUBY) $(WCONRAD_ASM) $<

.PHONY: clean
clean:
	rm -f $(WS)
