-include ../../config.mk
-include ../../config.tmpl.mk

ASM = $(shell find . -name '*.asm' -o -name '*.bak')
WS = $(patsubst %.asm, %.ws, $(patsubst %.bak, %.ws, $(ASM)))

.PHONY: all
all: $(WS)

%.ws: %.asm
	$(PALAIOLOGOS_WSI) --masm $< > $@
%.ws: %.bak
	$(PALAIOLOGOS_WSI) --masm $< > $@

.PHONY: clean
clean:
	rm -f $(WS)
