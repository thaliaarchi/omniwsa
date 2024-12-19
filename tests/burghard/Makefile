include ../base.mk

PASS = $(shell find pass -name '*.wsa')
FAIL = $(shell find fail -name '*.wsa')
WS = $(PASS:.wsa=.ws)
ERR = $(FAIL:.wsa=.err)

.PHONY: all
all: $(WS) $(ERR) wild.all

%.ws: %.wsa
	$(BURGHARD_WSA) --ext-syntax --ended $< > $@

%.err: %.wsa
	! $(BURGHARD_WSA) --ext-syntax --ended $< 2> $@

.PHONY: clean
clean: wild.clean
	@echo rm -f '**.ws' '**.err'
	@rm -f $(WS) $(ERR)

.PHONY: wild.all
wild.all:
	@echo make -C wild all
	@$(MAKE) -C wild all

.PHONY: wild.clean
wild.clean:
	@echo make -C wild clean
	@$(MAKE) -C wild clean
