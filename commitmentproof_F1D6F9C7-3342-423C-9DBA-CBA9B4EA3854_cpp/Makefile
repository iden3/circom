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
	
all: commitmentproof_F1D6F9C7-3342-423C-9DBA-CBA9B4EA3854
	
%.o: %.cpp $(DEPS_HPP)
	$(CC) -c $< $(CFLAGS)

fr_asm.o: fr.asm
	$(NASM) fr.asm -o fr_asm.o
	
commitmentproof_F1D6F9C7-3342-423C-9DBA-CBA9B4EA3854: $(DEPS_O) commitmentproof_F1D6F9C7-3342-423C-9DBA-CBA9B4EA3854.o
	$(CC) -o commitmentproof_F1D6F9C7-3342-423C-9DBA-CBA9B4EA3854 *.o -lgmp 
