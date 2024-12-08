-include ../config.mk
-include ../config.tmpl.mk

PASS = $(shell find pass wild -name '*.wsa')
FAIL = $(shell find fail -name '*.wsa')
WS = $(PASS:.wsa=.ws)
ERR = $(FAIL:.wsa=.err)
OPTIONS :=

.PHONY: all
all: $(WS) $(ERR)

%.extensions.ws: OPTIONS := --extensions
%.extensions.err: OPTIONS := --extensions

%.ws: %.wsa
	$(NODE) $(VOLIVA_CLI) compile $(OPTIONS) $< $@

%.err: %.wsa
	! $(NODE) $(VOLIVA_CLI) compile $(OPTIONS) $< $*.ws 2> $@ >&2
	@sed -i.bak "s|$$(realpath $(VOLIVA_WSA_DIR))|"'$$WSA|' $@
	@rm $@.bak

.PHONY: clean
clean:
	@echo rm -f '**.ws' '**.err'
	@rm -f $(WS) $(ERR)
