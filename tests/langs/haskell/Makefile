include ../../base.mk

SOURCES = $(shell find . -name '*.hs')
EXE = $(SOURCES:.hs=)
OUT = $(SOURCES:.hs=.out)

.PHONY: all
all: $(OUT)

$(EXE): %: %.hs
	$(GHC) $<

%.out: %
	$(GHC) --version > $@
	./$< >> $@

.PHONY: clean
clean:
	rm -f *.o *.hi $(EXE) $(OUT)
