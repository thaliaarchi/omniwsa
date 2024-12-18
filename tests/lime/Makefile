include ../base.mk

PASS = $(filter-out $(WILD_FAIL), $(shell find pass wild -name '*.wsa'))
FAIL = $(shell find fail -name '*.wsa') $(WILD_FAIL)
WILD_FAIL = wild/problem08.wsa wild/problem08_o.wsa
WS = $(PASS:.wsa=.ws)
ERR = $(FAIL:.wsa=.err)

.PHONY: all
all: $(WS) $(ERR)

# lwsa requires modifications to be able to specify input and output filenames
# that aren't in ws/ws/ and ws/wsa/, respectively.
%.ws: %.wsa
	$(CHRONIC) $(LIME_LWSA) $< $@ >$*.out 2>$*.err
	@! test -s $*.out && rm $*.out || echo 'Unexpected stdout'
	@! test -s $*.err && rm $*.err || echo 'Unexpected stderr'

%.err: %.wsa
	! $(LIME_LWSA) $< $*.err.ws > $@ 2>&1
	@! test -s $*.err.ws && rm $*.err.ws || echo 'Unexpected Whitespace output'

.PHONY: clean
clean:
	@echo rm -f '**.ws' '**.err'
	@rm -f $(WS) $(ERR)
