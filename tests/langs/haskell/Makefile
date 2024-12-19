include ../../base.mk

EXECUTE = $(shell find execute -name '*.hs')
COMPILE_PASS = $(shell find compile_pass -name '*.hs')
COMPILE_FAIL = $(shell find compile_fail -name '*.hs')
EXE = $(EXECUTE:.hs=) $(COMPILE_PASS:.hs=)
OUT = $(EXECUTE:.hs=.out)
ERR = $(COMPILE_FAIL:.hs=.err)

.PHONY: all
all: $(EXE) $(OUT) $(ERR)

$(EXE): %: %.hs
	$(GHC) $<

%.out: %
	$(GHC) --version > $@
	./$< >> $@

%.err: %.hs
	$(GHC) --version > $@
	! $(GHC) $< 2>> $@ >/dev/null

.PHONY: clean
clean:
	rm -f $(EXE) $(OUT) $(ERR) ghc.version
	find . \( -name '*.o' -o -name '*.hi' \) -delete
