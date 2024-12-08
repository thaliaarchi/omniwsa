SELF_DIR := $(dir $(lastword $(MAKEFILE_LIST)))

-include $(SELF_DIR)config.mk
include $(SELF_DIR)config.tmpl.mk
