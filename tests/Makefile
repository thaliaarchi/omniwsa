SUBDIRS := burghard censoredusername palaiologos voliva wconrad wsf \
	third_party/palaiologos third_party/voliva third_party/wconrad

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
