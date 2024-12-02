NODE = node
WSA_DIR = $(error 'Specify $$WSA_DIR')
CLI = $(WSA_DIR)/dist/cli.js

PASS = $(shell find pass -name '*.wsa')
FAIL = $(shell find fail -name '*.wsa')
WS = $(PASS:.wsa=.ws)
ERR = $(FAIL:.wsa=.err)

.PHONY: all
all: $(WS) $(ERR)

%.ws: %.wsa
	$(NODE) $(CLI) compile $< $@

%.err: %.wsa
	( ! $(NODE) $(CLI) compile $< $(<:.wsa=.ws) 2>&1 ) | sed "s|$$(realpath $(WSA_DIR))|"'$$WSA|' > $@

.PHONY: clean
clean:
	rm -f $(WS) $(ERR)
