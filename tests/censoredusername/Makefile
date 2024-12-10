include ../base.mk

PASS = $(shell find pass -name '*.wsa')
FAIL = $(shell find fail -name '*.wsa')
WS = $(PASS:.wsa=.ws)
ERR = $(FAIL:.wsa=.err)

.PHONY: all
all: $(WS) $(ERR)

%.ws: %.wsa
	$(CENSOREDUSERNAME_WSC) -t -f asm -o $@ $<

%.err: %.wsa
	! $(CENSOREDUSERNAME_WSC) -t -f asm -o $*.ws $< 2> $@

.PHONY: all
clean:
	@echo rm -f '**.ws' '**.err'
	@rm -f $(WS) $(ERR)
