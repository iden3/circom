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
	
all: commitmentproof_B261398E-2D93-4B1E-B349-E8CB2EF4C5D2
	
%.o: %.cpp $(DEPS_HPP)
	$(CC) -c $< $(CFLAGS)

fr_asm.o: fr.asm
	$(NASM) fr.asm -o fr_asm.o
	
commitmentproof_B261398E-2D93-4B1E-B349-E8CB2EF4C5D2: $(DEPS_O) commitmentproof_B261398E-2D93-4B1E-B349-E8CB2EF4C5D2.o
	$(CC) -o commitmentproof_B261398E-2D93-4B1E-B349-E8CB2EF4C5D2 *.o -lgmp 
