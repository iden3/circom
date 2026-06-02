CC=g++
CFLAGS=-std=c++11 -O3 -I.
GMP_CFLAGS=
DEPS_HPP = circom.hpp calcwit.hpp fr.hpp
DEPS_O = main.o calcwit.o fr.o 

ifeq ($(shell uname),Darwin)
	GMP_CFLAGS=-D_LONG_LONG_LIMB
endif

all: {{run_name}}

%.o: %.cpp $(DEPS_HPP)
	$(CC) -Wno-address-of-packed-member -c $< $(CFLAGS) $(GMP_CFLAGS)

{{run_name}}: $(DEPS_O) {{run_name}}.o
	$(CC) -o {{run_name}} *.o -lgmp {{#if has_parallelism}}-pthread{{/if}}
