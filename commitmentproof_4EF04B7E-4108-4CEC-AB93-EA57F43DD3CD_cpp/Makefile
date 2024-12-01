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
	
all: commitmentproof_4EF04B7E-4108-4CEC-AB93-EA57F43DD3CD
	
%.o: %.cpp $(DEPS_HPP)
	$(CC) -c $< $(CFLAGS)

fr_asm.o: fr.asm
	$(NASM) fr.asm -o fr_asm.o
	
commitmentproof_4EF04B7E-4108-4CEC-AB93-EA57F43DD3CD: $(DEPS_O) commitmentproof_4EF04B7E-4108-4CEC-AB93-EA57F43DD3CD.o
	$(CC) -o commitmentproof_4EF04B7E-4108-4CEC-AB93-EA57F43DD3CD *.o -lgmp 
