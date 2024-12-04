RUBY = ruby
ASM = $(error 'Specify $$ASM')

WSA = $(shell find . -name '*.wsa')
WS = $(WSA:.wsa=.ws)

.PHONY: all
all: $(WS)

%.ws: %.wsa
	$(RUBY) $(ASM) $<

.PHONY: clean
clean:
	rm -f $(WS)
