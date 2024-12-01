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
	
all: commitmentproof_DE30E936-E5B3-467D-ABDF-16FA4DC3C0CE
	
%.o: %.cpp $(DEPS_HPP)
	$(CC) -c $< $(CFLAGS)

fr_asm.o: fr.asm
	$(NASM) fr.asm -o fr_asm.o
	
commitmentproof_DE30E936-E5B3-467D-ABDF-16FA4DC3C0CE: $(DEPS_O) commitmentproof_DE30E936-E5B3-467D-ABDF-16FA4DC3C0CE.o
	$(CC) -o commitmentproof_DE30E936-E5B3-467D-ABDF-16FA4DC3C0CE *.o -lgmp 
