# Copy this template to config.mk to configure paths to each of these
# assemblers.

# Whitespace assemblers and tools
ALBINO ?= $(error 'Configure $$ALBINO in config.mk')
ALBINO_BUILD ?= $(ALBINO) build
ALBINO_EXEC ?= $(ALBINO) exec
ALBINO_GEN ?= $(ALBINO) gen
ALBINO_RUN ?= $(ALBINO) run
BURGHARD_WSA ?= $(error 'Configure $$BURGHARD_WSA in config.mk')
CENSOREDUSERNAME_WSC ?= $(error 'Configure $$CENSOREDUSERNAME_WSC in config.mk')
ESOTOPE_WS ?= $(error 'Configure $$ESOTOPE_WS in config.mk')
LIME_LWSA ?= $(error 'Configure $$LIME_LWSA in config.mk')
ICZELIA_WSI ?= $(error 'Configure $$ICZELIA_WSI in config.mk')
VOLIVA_DIR ?= $(error 'Configure $$VOLIVA_DIR in config.mk')
VOLIVA_CLI ?= $(VOLIVA_DIR)/dist/cli.js
WCONRAD_ASM ?= $(error 'Configure $$WCONRAD_ASM in config.mk')
WSF_ASSEMBLE ?= $(error 'Configure $$WSF_ASSEMBLE in config.mk')
WSF_WSC ?= $(CENSOREDUSERNAME_WSC)

# Languages
CC ?= gcc
GHC ?= ghc
MCS ?= mcs
MONO ?= mono
NODE ?= node
PYTHON2 ?= pyenv exec python2
PYTHON3 ?= python3
RUBY ?= ruby

# Shell tools
CHRONIC ?= chronic # from moreutils
TIMEOUT_SECS ?= 1
TIMEOUT ?= timeout $(TIMEOUT_SECS) # from coreutils
