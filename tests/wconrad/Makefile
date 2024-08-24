ASM = ruby "$$(which whitespace-asm)"

WSA = $(shell find . -name '*.wsa')
WS = $(patsubst %.wsa, %.ws, $(WSA))

.PHONY: all
all: $(WS)

%.ws: %.wsa
	$(ASM) $<

.PHONY: clean
clean:
	rm -f $(WS)
