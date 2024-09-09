NODE = node
CLI = dist/cli.js

PASS = $(shell find . -name '*.wsa' -a ! -name count.wsa)
FAIL = wsa-tests/count.wsa
WS = $(PASS:.wsa=.ws)
ERR = $(FAIL:.wsa=.err)

.PHONY: all
all: $(WS) $(ERR)

%.extensions.ws: %.extensions.wsa
	$(NODE) $(CLI) compile --debugger $< $@

%.ws: %.wsa
	$(NODE) $(CLI) compile $< $@

%.err: %.wsa
	! $(NODE) $(CLI) compile $< $(<:.wsa=.ws) 2> $@

.PHONY: clean
clean:
	rm -f $(WS) $(ERR)
