CC=g++
CFLAGS=-std=c++11 -O3 -I.
DEPS_HPP = circom.hpp calcwit.hpp fr.hpp
DEPS_O = main.o calcwit.o

all: {{run_name}}

%.o: %.cpp $(DEPS_HPP)
	$(CC) -c $< $(CFLAGS)

{{run_name}}: $(DEPS_O) {{run_name}}.o
	$(CC) -o {{run_name}} *.o -lgmp {{#if has_parallelism}}-pthread{{/if}}
