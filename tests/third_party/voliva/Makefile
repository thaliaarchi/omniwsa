NODE = node
WSA_DIR = $(error 'Specify $$WSA_DIR')
CLI = $(WSA_DIR)/dist/cli.js

PASS = $(filter-out $(FAIL), $(shell find . -name '*.wsa'))
FAIL =
WS = $(PASS:.wsa=.ws)
ERR = $(FAIL:.wsa=.err)

.PHONY: all
all: $(WS) $(ERR)

%.extensions.ws: %.extensions.wsa
	$(NODE) $(CLI) compile --extensions $< $@

%.ws: %.wsa
	$(NODE) $(CLI) compile $< $@

%.err: %.wsa
	! $(NODE) $(CLI) compile $< $(<:.wsa=.ws) 2> $@ >&2
	@sed -i.bak "s|$$(realpath $(WSA_DIR))|"'$$WSA|' $@
	@rm $@.bak

.PHONY: clean
clean:
	rm -f $(WS) $(ERR)
