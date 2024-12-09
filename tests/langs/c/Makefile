include ../../base.mk

SOURCES = $(shell find . -name '*.c')
EXE = $(SOURCES:.c=)
OUT = $(SOURCES:.c=.out)

.PHONY: all
all: $(OUT)

$(EXE): %: %.c
	$(CC) -o $@ $<

%.out: %
	$(CC) --version | head -n1 > $@
	./$< >> $@

.PHONY: clean
clean:
	rm -f *.o $(EXE) $(OUT)
