-include ../../config.mk
-include ../../config.tmpl.mk

PASS = $(filter-out $(FAIL), $(shell find . -name '*.wsa'))
FAIL =
WS = $(PASS:.wsa=.ws)
ERR = $(FAIL:.wsa=.err)

.PHONY: all
all: $(WS) $(ERR)

%.extensions.ws: %.extensions.wsa
	$(NODE) $(VOLIVA_CLI) compile --extensions $< $@

%.ws: %.wsa
	$(NODE) $(VOLIVA_CLI) compile $< $@

%.err: %.wsa
	! $(NODE) $(VOLIVA_CLI) compile $< $(<:.wsa=.ws) 2> $@ >&2
	@sed -i.bak "s|$$(realpath $(VOLIVA_WSA_DIR))|"'$$WSA|' $@
	@rm $@.bak

.PHONY: clean
clean:
	rm -f $(WS) $(ERR)
