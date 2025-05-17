FLAGS = -g -O0 -ffreestanding -nostdlib -fpic

all: init
	nasm -f elf halt.asm -o build/halt.o
	i686-elf-gcc $(FLAGS) -T bin.ld build/halt.o -o build/halt

clean:
	rm -rf build

init:
	mkdir -p build
