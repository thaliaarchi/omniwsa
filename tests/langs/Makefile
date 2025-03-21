SUBDIRS := c csharp haskell javascript python2 python3 ruby

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
