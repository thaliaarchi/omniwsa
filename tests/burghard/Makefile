WSA = wsa

PASS = $(shell find pass -name '*.wsa')
FAIL = $(shell find fail -name '*.wsa')
WS = $(patsubst %.wsa, %.ws, $(PASS))
ERR = $(patsubst %.wsa, %.err, $(FAIL))

.PHONY: all
all: $(WS) $(ERR)

%.ws: %.wsa
	$(WSA) --ext-syntax --ended $< > $@

%.err: %.wsa
	! $(WSA) --ext-syntax --ended $< 2> $@

.PHONY: clean
clean:
	rm -f $(WS) $(ERR)
