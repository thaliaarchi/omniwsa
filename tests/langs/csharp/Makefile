include ../../base.mk

SOURCES = $(shell find . -name '*.cs')
EXE = $(SOURCES:.cs=.exe)
OUT = $(SOURCES:.cs=.out)

.PHONY: all
all: $(OUT)

%.exe: %.cs
	$(MCS) -out:$@ $<

%.out: %.exe
	$(MCS) --version > $@
	$(MONO) $< >> $@

.PHONY: clean
clean:
	rm -f *.o $(EXE) $(OUT)
