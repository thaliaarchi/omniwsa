-include ../config.mk
-include ../config.tmpl.mk

PASS = $(shell find pass -name '*.wsa')
FAIL = $(shell find fail -name '*.wsa')
WS = $(PASS:.wsa=.ws)
ERR = $(FAIL:.wsa=.err)

.PHONY: all
all: $(WS) $(ERR)

%.ws: %.wsa
	$(NODE) $(VOLIVA_CLI) compile --extensions $< $@

%.err: %.wsa
	! $(NODE) $(VOLIVA_CLI) compile --extensions $< $(<:.wsa=.ws) 2> $@ >&2
	@sed -i.bak "s|$$(realpath $(VOLIVA_WSA_DIR))|"'$$WSA|' $@
	@rm $@.bak

.PHONY: clean
clean:
	rm -f $(WS) $(ERR)
