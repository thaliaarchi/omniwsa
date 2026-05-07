include ../base.mk

PASS = $(shell find pass wild -name '*.asm' -o -name '*.bak')
FAIL = $(shell find fail -name '*.asm' -o -name '*.bak')
TIMEOUT_ASM = $(shell find timeout -name '*.asm' -o -name '*.bak')
WS = $(patsubst %.asm, %.ws, $(patsubst %.bak, %.ws, $(PASS)))
ERR = $(patsubst %.asm, %.err, $(patsubst %.bak, %.err, $(FAIL)))
TIMEOUT_OUT = $(patsubst %.asm, %.timeout, $(patsubst %.bak, %.timeout, $(TIMEOUT_ASM)))

.PHONY: all
all: $(WS) $(ERR) $(TIMEOUT_OUT)

%.ws: %.asm
	$(ICZELIA_WSI) --masm $< > $@
%.ws: %.bak
	$(ICZELIA_WSI) --masm $< > $@

%.err: %.asm
	! $(ICZELIA_WSI) --masm $< 2> $@
%.err: %.bak
	! $(ICZELIA_WSI) --masm $< 2> $@

%.timeout: %.asm
	! $(TIMEOUT) $(ICZELIA_WSI) --masm $< >/dev/null && touch $@
%.timeout: %.bak
	! $(TIMEOUT) $(ICZELIA_WSI) --masm $< >/dev/null && touch $@

.PHONY: clean
clean:
	@echo rm -f '**.ws' '**.err' '**.timeout'
	@rm -f $(WS) $(ERR) $(TIMEOUT_OUT)
