WSA = $(error 'Specify $$WSA')

PASS = $(shell find pass -name '*.wsa')
FAIL = $(shell find fail -name '*.wsa')
WS = $(PASS:.wsa=.ws)
ERR = $(FAIL:.wsa=.err)

.PHONY: all
all: $(WS) $(ERR)

%.ws: %.wsa
	$(WSA) --ext-syntax --ended $< > $@

%.err: %.wsa
	! $(WSA) --ext-syntax --ended $< 2> $@

.PHONY: clean
clean:
	rm -f $(WS) $(ERR)
