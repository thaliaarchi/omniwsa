WSF = wsf-assemble
WSC = wsc-fork

PASS = $(shell find pass -name '*.wsf')
FAIL = $(shell find fail -name '*.wsf')
WSA = $(PASS:.wsf=.wsa) $(FAIL:.wsf=.wsa)
WS = $(PASS:.wsf=.ws)
ERR = $(FAIL:.wsf=.err)

.PHONY: all
all: $(WSA) $(WS) $(ERR)

%.wsa: %.wsf
	$(WSF) $< $@

%.ws: %.wsa
	$(WSC) -t -f asm -o $@ $<

%.err: %.wsa
	! $(WSC) -t -f asm $< 2> $@

.PHONY: clean
clean:
	rm -f $(WSA) $(WS) $(ERR)
