CC=g++
CFLAGS=-std=c++11 -O3 -I.
DEPS_HPP = circom.hpp calcwit.hpp fr.hpp
DEPS_O = main.o calcwit.o fr.o fr_asm.o

ifeq ($(shell uname),Darwin)
	NASM=nasm -fmacho64 --prefix _
endif
ifeq ($(shell uname),Linux)
	NASM=nasm -felf64
endif
	
all: commitmentproof_5E28DACE-D540-4C2A-8155-7D6C838A0184
	
%.o: %.cpp $(DEPS_HPP)
	$(CC) -c $< $(CFLAGS)

fr_asm.o: fr.asm
	$(NASM) fr.asm -o fr_asm.o
	
commitmentproof_5E28DACE-D540-4C2A-8155-7D6C838A0184: $(DEPS_O) commitmentproof_5E28DACE-D540-4C2A-8155-7D6C838A0184.o
	$(CC) -o commitmentproof_5E28DACE-D540-4C2A-8155-7D6C838A0184 *.o -lgmp 
