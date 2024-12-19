include ../../base.mk

WSA = io.wsa memory.wsa prim.wsa
WS = $(WSA:.wsa=.ws) \
	$(foreach config, \
		none h s sh t th ts tsh c ch cs csh ct cth cts ctsh \
		e eh es esh et eth ets etsh ec ech ecs ecsh ect ecth ects ectsh, \
		wsinterws.$(config).ws)
PWS = wsinterws.none.pws # Only one PWS file is sufficient to test the syntax.

.PHONY: all
all: $(WS) $(PWS)
	@bash -c 'diff <(dos2unix < wsinterws.origin.ws) wsinterws.none.ws' || echo 'wsinterws.none.ws differs from wsinterws.origin.ws'

# --ended is used for all targets, as it was originally always enabled and only
# became configurable with the new CLI.

%.ws: %.wsa
	$(BURGHARD_WSA) --ended --ext-syntax $< > $@

wsinterws.%.ws: wsinterws.wsa
	$(BURGHARD_WSA) --ended $(WSA_OPTIONS) $< > $@
wsinterws.%.pws: wsinterws.wsa
	$(BURGHARD_WSA) --ended --pws $(WSA_OPTIONS) $< > $@

wsinterws.none.%:  WSA_OPTIONS :=
wsinterws.h.%:     WSA_OPTIONS :=                                                                 -o trace_heap
wsinterws.s.%:     WSA_OPTIONS :=                                                  -o trace_stack
wsinterws.sh.%:    WSA_OPTIONS :=                                                  -o trace_stack -o trace_heap
wsinterws.t.%:     WSA_OPTIONS :=                                   -o print_trace
wsinterws.th.%:    WSA_OPTIONS :=                                   -o print_trace                -o trace_heap
wsinterws.ts.%:    WSA_OPTIONS :=                                   -o print_trace -o trace_stack
wsinterws.tsh.%:   WSA_OPTIONS :=                                   -o print_trace -o trace_stack -o trace_heap
wsinterws.c.%:     WSA_OPTIONS :=              -o print_compilation
wsinterws.ch.%:    WSA_OPTIONS :=              -o print_compilation                               -o trace_heap
wsinterws.cs.%:    WSA_OPTIONS :=              -o print_compilation                -o trace_stack
wsinterws.csh.%:   WSA_OPTIONS :=              -o print_compilation                -o trace_stack -o trace_heap
wsinterws.ct.%:    WSA_OPTIONS :=              -o print_compilation -o print_trace
wsinterws.cth.%:   WSA_OPTIONS :=              -o print_compilation -o print_trace                -o trace_heap
wsinterws.cts.%:   WSA_OPTIONS :=              -o print_compilation -o print_trace -o trace_stack
wsinterws.ctsh.%:  WSA_OPTIONS :=              -o print_compilation -o print_trace -o trace_stack -o trace_heap
wsinterws.e.%:     WSA_OPTIONS := --ext-syntax
wsinterws.eh.%:    WSA_OPTIONS := --ext-syntax                                                    -o trace_heap
wsinterws.es.%:    WSA_OPTIONS := --ext-syntax                                     -o trace_stack
wsinterws.esh.%:   WSA_OPTIONS := --ext-syntax                                     -o trace_stack -o trace_heap
wsinterws.et.%:    WSA_OPTIONS := --ext-syntax                      -o print_trace
wsinterws.eth.%:   WSA_OPTIONS := --ext-syntax                      -o print_trace                -o trace_heap
wsinterws.ets.%:   WSA_OPTIONS := --ext-syntax                      -o print_trace -o trace_stack
wsinterws.etsh.%:  WSA_OPTIONS := --ext-syntax                      -o print_trace -o trace_stack -o trace_heap
wsinterws.ec.%:    WSA_OPTIONS := --ext-syntax -o print_compilation
wsinterws.ech.%:   WSA_OPTIONS := --ext-syntax -o print_compilation                               -o trace_heap
wsinterws.ecs.%:   WSA_OPTIONS := --ext-syntax -o print_compilation                -o trace_stack
wsinterws.ecsh.%:  WSA_OPTIONS := --ext-syntax -o print_compilation                -o trace_stack -o trace_heap
wsinterws.ect.%:   WSA_OPTIONS := --ext-syntax -o print_compilation -o print_trace
wsinterws.ecth.%:  WSA_OPTIONS := --ext-syntax -o print_compilation -o print_trace                -o trace_heap
wsinterws.ects.%:  WSA_OPTIONS := --ext-syntax -o print_compilation -o print_trace -o trace_stack
wsinterws.ectsh.%: WSA_OPTIONS := --ext-syntax -o print_compilation -o print_trace -o trace_stack -o trace_heap

.PHONY: clean
clean:
	@echo rm -f '**.ws' '**.pws'
	@rm -f $(WS) $(PWS)
