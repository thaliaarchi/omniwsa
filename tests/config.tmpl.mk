# Copy this template to config.mk to configure paths to each of these
# assemblers.

BURGHARD_WSA ?= $(error 'Configure $$BURGHARD_WSA in config.mk')
CENSOREDUSERNAME_WSC ?= $(error 'Configure $$CENSOREDUSERNAME_WSC in config.mk')
PALAIOLOGOS_WSI ?= $(error 'Configure $$PALAIOLOGOS_WSI in config.mk')
VOLIVA_WSA_DIR ?= $(error 'Configure $$VOLIVA_WSA_DIR in config.mk')
VOLIVA_CLI ?= $(VOLIVA_WSA_DIR)/dist/cli.js
WCONRAD_ASM ?= $(error 'Configure $$WCONRAD_ASM in config.mk')
WSF_ASSEMBLE ?= $(error 'Configure $$WSF_ASSEMBLE in config.mk')
WSF_WSC ?= $(CENSOREDUSERNAME_WSC)

CC ?= gcc
GHC ?= ghc
NODE ?= node
RUBY ?= ruby

TIMEOUT_SECS ?= 1
