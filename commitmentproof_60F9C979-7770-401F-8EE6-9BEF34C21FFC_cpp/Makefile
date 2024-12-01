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
	
all: commitmentproof_60F9C979-7770-401F-8EE6-9BEF34C21FFC
	
%.o: %.cpp $(DEPS_HPP)
	$(CC) -c $< $(CFLAGS)

fr_asm.o: fr.asm
	$(NASM) fr.asm -o fr_asm.o
	
commitmentproof_60F9C979-7770-401F-8EE6-9BEF34C21FFC: $(DEPS_O) commitmentproof_60F9C979-7770-401F-8EE6-9BEF34C21FFC.o
	$(CC) -o commitmentproof_60F9C979-7770-401F-8EE6-9BEF34C21FFC *.o -lgmp 
