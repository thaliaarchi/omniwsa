include ../base.mk

PASS = $(shell find pass -name '*.asm')
FAIL = $(shell find fail -name '*.asm')
BC = $(PASS:.asm=.bc)
WS = $(PASS:.asm=.ws)
DISASM = $(PASS:.asm=.disasm)
ERR = $(FAIL:.asm=.err)

.PHONY: all
all: $(BC) $(WS) $(DISASM) $(ERR)

%.bc: %.asm
	$(ALBINO_BUILD) -s asm -o $@ $<

%.ws: %.bc
	$(ALBINO_GEN) -s ws -o $@ $<

%.disasm: %.bc
	$(ALBINO_GEN) -s asm -o $@ $<

%.err: %.asm
	! $(ALBINO_BUILD) -s asm -o $*.bc $< 2> $@ >&2
	@rm $*.bc

.PHONY: all
clean:
	@echo rm -f '**.bc' '**.ws' '**.disasm' '**.err'
	@rm -f $(BC) $(WS) $(DISASM) $(FAIL:.asm=.bc) $(ERR)
