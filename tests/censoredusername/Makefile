include ../base.mk

PASS = $(shell find pass -name '*.wsa')
FAIL = $(shell find fail -name '*.wsa')
WS = $(PASS:.wsa=.ws)
MIN_WS = pass/regress/label_sort_unstable.min.ws
ERR = $(FAIL:.wsa=.err)

.PHONY: all
all: $(WS) $(MIN_WS) $(ERR)

%.ws: %.wsa
	$(CENSOREDUSERNAME_WSC) --translate --format asm -o $@ $<

%.min.ws: %.wsa
	$(CENSOREDUSERNAME_WSC) --translate --format asm --minify -o $@ $<

%.err: %.wsa
	! $(CENSOREDUSERNAME_WSC) --translate --format asm -o $*.ws $< 2> $@

.PHONY: all
clean:
	@echo rm -f '**.ws' '**.err'
	@rm -f $(WS) $(MIN_WS) $(ERR)
