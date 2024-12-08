-include ../config.mk
-include ../config.tmpl.mk

PASS = $(shell find pass -name '*.wsa')
FAIL = $(shell find fail -name '*.wsa')
WS = $(PASS:.wsa=.ws)
ERR = $(FAIL:.wsa=.err)

.PHONY: all
all: $(WS) $(ERR)

%.ws: %.wsa
	$(CENSOREDUSERNAME_WSC) -t -f asm -o $@ $<

# Test for empty stdout, since wsc always exits with 0 unless it panics.
%.err: %.wsa
	! $(CENSOREDUSERNAME_WSC) -t -f asm $< 2> $@ > $<.out || ! test -s $<.out && rm $<.out

.PHONY: all
clean:
	@echo rm -f '**.ws' '**.err'
	@rm -f $(WS) $(ERR)
