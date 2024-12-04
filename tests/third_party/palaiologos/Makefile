WSI = $(error 'Specify $$WSI')

ASM = $(shell find . -name '*.asm' -o -name '*.bak')
WS = $(patsubst %.asm, %.ws, $(patsubst %.bak, %.ws, $(ASM)))

.PHONY: all
all: $(WS)

%.ws: %.asm
	$(WSI) --masm $< > $@
%.ws: %.bak
	$(WSI) --masm $< > $@

.PHONY: clean
clean:
	rm -f $(WS)
