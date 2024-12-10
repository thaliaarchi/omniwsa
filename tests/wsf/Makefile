include ../base.mk

PASS = $(shell find pass -name '*.wsf')
FAIL = $(shell find fail -name '*.wsf')
WSA = $(PASS:.wsf=.wsa) $(FAIL:.wsf=.wsa)
WS = $(PASS:.wsf=.ws)
ERR = $(FAIL:.wsf=.err)

.PHONY: all
all: $(WSA) $(WS) $(ERR)

%.wsa: %.wsf
	$(WSF_ASSEMBLE) $< $@

%.ws: %.wsa
	$(WSF_WSC) -t -f asm -o $@ $<

%.err: %.wsa
	! $(WSF_WSC) -t -f asm -o $*.ws $< 2> $@

fail/hash_in_import.wsa fail/hash_in_export.wsa: fail/path\#with_hash.wsf

.INTERMEDIATE: fail/path\#with_hash.wsf
fail/path\#with_hash.wsf:
	touch $@

.PHONY: clean
clean:
	@echo rm -f '**.wsa' '**.ws' '**.err'
	@rm -f $(WSA) $(WS) $(ERR)
