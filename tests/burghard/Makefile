include ../base.mk

PASS = $(shell find pass -name '*.wsa')
FAIL = $(shell find fail -name '*.wsa')
WS = $(PASS:.wsa=.ws)
ERR = $(FAIL:.wsa=.err)

.PHONY: all
all: $(WS) $(ERR)

%.ws: %.wsa
	$(BURGHARD_WSA) --ext-syntax --ended $< > $@

%.err: %.wsa
	! $(BURGHARD_WSA) --ext-syntax --ended $< 2> $@

.PHONY: clean
clean:
	@echo rm -f '**.ws' '**.err'
	@rm -f $(WS) $(ERR)
