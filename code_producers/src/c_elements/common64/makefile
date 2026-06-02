CC=g++
CFLAGS=-std=c++11 -O3 -I.
GMP_CFLAGS=
EXTRA_FILES = json2bin64.cpp
DEPS_HPP = circom.hpp calcwit.hpp fr.hpp
DEPS_CPP := $(filter-out $(EXTRA_FILES), $(wildcard *.cpp))
DEPS_O := $(patsubst %.cpp,%.o,$(DEPS_CPP))

ifeq ($(shell uname),Darwin)
	GMP_CFLAGS=-D_LONG_LONG_LIMB
endif

all: {{run_name}}

%.o: %.cpp $(DEPS_HPP)
	$(CC) -c $< $(CFLAGS) $(GMP_CFLAGS)

{{run_name}}: $(DEPS_O) {{run_name}}.o
	$(CC) -o {{run_name}} $(DEPS_O) -lgmp {{#if has_parallelism}}-pthread{{/if}}

json2bin64: json2bin64.cpp
	$(CC) -o json2bin64 json2bin64.cpp $(GMP_CFLAGS) -lgmp
