CC=g++
CFLAGS=-std=c++11 -O3 -I.
GMP_CFLAGS=
DEPS_HPP = circom.hpp calcwit.hpp fr.hpp
DEPS_O = main.o calcwit.o fr.o fr_asm.o

ifeq ($(shell uname),Darwin)
	GMP_CFLAGS=-D_LONG_LONG_LIMB
	NASM=nasm -fmacho64 --prefix _
endif
ifeq ($(shell uname),Linux)
	NASM=nasm -felf64
endif
	
all: {{run_name}}
	
%.o: %.cpp $(DEPS_HPP)
	$(CC) -c $< $(CFLAGS) $(GMP_CFLAGS)

fr_asm.o: fr.asm
	$(NASM) fr.asm -o fr_asm.o
	
{{run_name}}: $(DEPS_O) {{run_name}}.o
	$(CC) -o {{run_name}} *.o -lgmp {{#if has_parallelism}}-pthread{{/if}}
