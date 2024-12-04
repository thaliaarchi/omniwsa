WSI = $(error 'Specify $$WSI')

PASS = $(shell find pass -name '*.asm' -o -name '*.bak')
FAIL = $(shell find fail -name '*.asm' -o -name '*.bak')
WS = $(patsubst %.asm, %.ws, $(patsubst %.bak, %.ws, $(PASS)))
ERR = $(patsubst %.asm, %.err, $(patsubst %.bak, %.err, $(FAIL)))

.PHONY: all
all: $(WS) $(ERR)

%.ws: %.asm
	$(WSI) --masm $< > $@

%.err: %.asm
	! $(WSI) --masm $< 2> $@

.PHONY: clean
clean:
	rm -f $(WS) $(ERR)
