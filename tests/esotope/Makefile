include ../base.mk

PASS = $(shell find pass -name '*.wsa')
FAIL = $(shell find fail -name '*.wsa')
WS = $(PASS:.wsa=.ws)
ERR = $(FAIL:.wsa=.err)

.PHONY: all
all: $(WS) $(ERR)

%.ws: %.wsa
	$(PYTHON2) $(ESOTOPE_WS) -c $< $@ 2> $*.err
	@! test -s $*.err && rm $*.err

%.err: %.wsa
	! $(PYTHON2) $(ESOTOPE_WS) -c $< $*.ws 2> $@
	@sed -i.bak "s|$$(dirname $$(realpath $(ESOTOPE_WS)))/||" $@
	@rm $@.bak
	@! test -s $*.ws && rm $*.ws

.PHONY: all
clean:
	@echo rm -f '**.ws' '**.err'
	@rm -f $(WS) $(ERR)
