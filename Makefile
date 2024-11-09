FLAGS = -Wall -Wextra -g \
		-O0 -fstrength-reduce -fomit-frame-pointer -finline-functions \
		-ffreestanding -nostdlib -nostartfiles -nodefaultlibs -fno-builtin \
		-falign-functions -falign-loops -falign-jumps -falign-labels

NFLAGS = -g

LDFLAGS = -g

all: init build/bootloader.bin build/kernel.bin
	cat build/bootloader.bin build/kernel.bin > build/ghostos.bin
	@if [ $$(stat -c '%s' build/ghostos.bin) -gt 51712 ]; then \
		echo "Kernel code exceeds 51712 bytes." >&2; \
		echo "Please check bootloader read parameters." >&2; \
		echo "Please check stack address." >&2; \
		exit 1; \
	fi
	truncate -s 51712 build/ghostos.bin

run: all
	qemu-system-x86_64 -hda build/ghostos.bin

debug: all
	gdb \
		-ex "set confirm off" \
		-ex "add-symbol-file build/kernel.tmp.o 0x100000" \
		-ex "break _start" \
		-ex "target remote | qemu-system-x86_64 -hda build/ghostos.bin -S -gdb stdio" \
		-ex "continue"

clean:
	rm -rf build

init:
	mkdir -p build

# Order of the files is important
build/kernel.bin: build/kernel.asm.o build/kernel.o build/hal.o build/memory.o build/console.o
	i686-elf-ld -relocatable $(LDFLAGS) build/kernel.asm.o build/kernel.o build/hal.o build/memory.o build/console.o -o build/kernel.tmp.o
	i686-elf-gcc $(FLAGS) -T linker.ld build/kernel.tmp.o -o build/kernel.bin -static-libgcc -lgcc

build/kernel.o: kernel.c
	i686-elf-gcc $(FLAGS) -c kernel.c -o build/kernel.o

build/kernel.asm.o: kernel.asm
	nasm -f elf $(NFLAGS) kernel.asm -o build/kernel.asm.o

build/hal.o: hal.c
	i686-elf-gcc $(FLAGS) -c hal.c -o build/hal.o

build/memory.o: memory.c
	i686-elf-gcc $(FLAGS) -c memory.c -o build/memory.o

build/console.o: console.c
	i686-elf-gcc $(FLAGS) -c console.c -o build/console.o

build/bootloader.bin: bootloader.asm
	nasm bootloader.asm -o build/bootloader.bin
