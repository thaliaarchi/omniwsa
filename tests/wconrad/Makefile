include ../base.mk

WSA = $(shell find . -name '*.wsa')
WS = $(WSA:.wsa=.ws)

.PHONY: all
all: $(WS)

%.ws: %.wsa
	$(RUBY) $(WCONRAD_ASM) $<

.PHONY: clean
clean:
	@echo rm -f '**.ws'
	@rm -f $(WS)
