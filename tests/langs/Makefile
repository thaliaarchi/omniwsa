SUBDIRS := c haskell javascript python2

.PHONY: all clean
all: $(SUBDIRS:%=%.all)
clean: $(SUBDIRS:%=%.clean)

.PHONY: %.all
%.all:
	@echo make -C $* all
	@$(MAKE) -C $* all

.PHONY: %.clean
%.clean:
	@echo make -C $* clean
	@$(MAKE) -C $* clean
