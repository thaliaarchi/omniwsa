WSC = $(error 'Specify $$WSC')

PASS = $(shell find pass -name '*.wsa')
FAIL = $(shell find fail -name '*.wsa')
WS = $(PASS:.wsa=.ws)
ERR = $(FAIL:.wsa=.err)

.PHONY: all
all: $(WS) $(ERR)

%.ws: %.wsa
	$(WSC) -t -f asm -o $@ $<

# Test for empty stdout, since wsc always exits with 0 unless it panics.
%.err: %.wsa
	! $(WSC) -t -f asm $< 2> $@ > $<.out || ! test -s $<.out && rm $<.out

.PHONY: all
clean:
	rm -f $(WS) $(ERR)
