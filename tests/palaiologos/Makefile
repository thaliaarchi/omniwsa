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
	$(PALAIOLOGOS_WSI) --masm $< > $@
%.ws: %.bak
	$(PALAIOLOGOS_WSI) --masm $< > $@

%.err: %.asm
	! $(PALAIOLOGOS_WSI) --masm $< 2> $@
%.err: %.bak
	! $(PALAIOLOGOS_WSI) --masm $< 2> $@

%.timeout: %.asm
	! $(TIMEOUT) $(PALAIOLOGOS_WSI) --masm $< >/dev/null && touch $@
%.timeout: %.bak
	! $(TIMEOUT) $(PALAIOLOGOS_WSI) --masm $< >/dev/null && touch $@

.PHONY: clean
clean:
	@echo rm -f '**.ws' '**.err' '**.timeout'
	@rm -f $(WS) $(ERR) $(TIMEOUT_OUT)
