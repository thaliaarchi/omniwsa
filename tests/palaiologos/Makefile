WSI = $(error 'Specify $$WSI')

PASS = $(shell find pass -name '*.asm' -o -name '*.bak')
FAIL = $(shell find fail -name '*.asm' -o -name '*.bak')
TIMEOUT_ASM = $(shell find timeout -name '*.asm' -o -name '*.bak')
WS = $(patsubst %.asm, %.ws, $(patsubst %.bak, %.ws, $(PASS)))
ERR = $(patsubst %.asm, %.err, $(patsubst %.bak, %.err, $(FAIL)))
TIMEOUT = $(patsubst %.asm, %.timeout, $(patsubst %.bak, %.timeout, $(TIMEOUT_ASM)))

.PHONY: all
all: $(WS) $(ERR) $(TIMEOUT)

%.ws: %.asm
	$(WSI) --masm $< > $@

%.err: %.asm
	! $(WSI) --masm $< 2> $@

%.timeout: %.asm
	! timeout 2 $(WSI) --masm $< >/dev/null && touch $@

.PHONY: clean
clean:
	rm -f $(WS) $(ERR) $(TIMEOUT)
