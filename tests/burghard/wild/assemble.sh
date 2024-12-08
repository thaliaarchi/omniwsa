#!/usr/bin/env bash
set -eEuo pipefail

wsa=burghard-wsa

# --ended is not configured here, because it is a modern modification that was
# not originally configurable.

mkdir generated

$wsa io.wsa     --ended > generated/io.ws
$wsa memory.wsa --ended > generated/memory.ws
$wsa prim.wsa   --ended > generated/prim.ws

$wsa wsinterws.wsa --ended                                                                               > generated/wsinterws_.ws
$wsa wsinterws.wsa --ended                                                                 -o trace_heap > generated/wsinterws_h.ws
$wsa wsinterws.wsa --ended                                                  -o trace_stack               > generated/wsinterws_s.ws
$wsa wsinterws.wsa --ended                                                  -o trace_stack -o trace_heap > generated/wsinterws_sh.ws
$wsa wsinterws.wsa --ended                                   -o print_trace                              > generated/wsinterws_t.ws
$wsa wsinterws.wsa --ended                                   -o print_trace                -o trace_heap > generated/wsinterws_th.ws
$wsa wsinterws.wsa --ended                                   -o print_trace -o trace_stack               > generated/wsinterws_ts.ws
$wsa wsinterws.wsa --ended                                   -o print_trace -o trace_stack -o trace_heap > generated/wsinterws_tsh.ws
$wsa wsinterws.wsa --ended              -o print_compilation                                             > generated/wsinterws_c.ws
$wsa wsinterws.wsa --ended              -o print_compilation                               -o trace_heap > generated/wsinterws_ch.ws
$wsa wsinterws.wsa --ended              -o print_compilation                -o trace_stack               > generated/wsinterws_cs.ws
$wsa wsinterws.wsa --ended              -o print_compilation                -o trace_stack -o trace_heap > generated/wsinterws_csh.ws
$wsa wsinterws.wsa --ended              -o print_compilation -o print_trace                              > generated/wsinterws_ct.ws
$wsa wsinterws.wsa --ended              -o print_compilation -o print_trace                -o trace_heap > generated/wsinterws_cth.ws
$wsa wsinterws.wsa --ended              -o print_compilation -o print_trace -o trace_stack               > generated/wsinterws_cts.ws
$wsa wsinterws.wsa --ended              -o print_compilation -o print_trace -o trace_stack -o trace_heap > generated/wsinterws_ctsh.ws
$wsa wsinterws.wsa --ended --ext-syntax                                                                  > generated/wsinterws_e.ws
$wsa wsinterws.wsa --ended --ext-syntax                                                    -o trace_heap > generated/wsinterws_eh.ws
$wsa wsinterws.wsa --ended --ext-syntax                                     -o trace_stack               > generated/wsinterws_es.ws
$wsa wsinterws.wsa --ended --ext-syntax                                     -o trace_stack -o trace_heap > generated/wsinterws_esh.ws
$wsa wsinterws.wsa --ended --ext-syntax                      -o print_trace                              > generated/wsinterws_et.ws
$wsa wsinterws.wsa --ended --ext-syntax                      -o print_trace                -o trace_heap > generated/wsinterws_eth.ws
$wsa wsinterws.wsa --ended --ext-syntax                      -o print_trace -o trace_stack               > generated/wsinterws_ets.ws
$wsa wsinterws.wsa --ended --ext-syntax                      -o print_trace -o trace_stack -o trace_heap > generated/wsinterws_etsh.ws
$wsa wsinterws.wsa --ended --ext-syntax -o print_compilation                                             > generated/wsinterws_ec.ws
$wsa wsinterws.wsa --ended --ext-syntax -o print_compilation                               -o trace_heap > generated/wsinterws_ech.ws
$wsa wsinterws.wsa --ended --ext-syntax -o print_compilation                -o trace_stack               > generated/wsinterws_ecs.ws
$wsa wsinterws.wsa --ended --ext-syntax -o print_compilation                -o trace_stack -o trace_heap > generated/wsinterws_ecsh.ws
$wsa wsinterws.wsa --ended --ext-syntax -o print_compilation -o print_trace                              > generated/wsinterws_ect.ws
$wsa wsinterws.wsa --ended --ext-syntax -o print_compilation -o print_trace                -o trace_heap > generated/wsinterws_ecth.ws
$wsa wsinterws.wsa --ended --ext-syntax -o print_compilation -o print_trace -o trace_stack               > generated/wsinterws_ects.ws
$wsa wsinterws.wsa --ended --ext-syntax -o print_compilation -o print_trace -o trace_stack -o trace_heap > generated/wsinterws_ectsh.ws

# Only one PWS file is sufficient to test the syntax.
$wsa wsinterws.wsa --ended --pws > generated/wsinterws_.pws

diff <(dos2unix < wsinterws.ws) generated/wsinterws_.ws
