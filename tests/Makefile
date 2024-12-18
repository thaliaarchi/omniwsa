SUBDIRS := burghard censoredusername esotope langs lime palaiologos voliva \
	wconrad wsf

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
