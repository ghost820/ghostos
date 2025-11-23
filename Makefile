FLAGS = -Wall -Wextra -g \
		-O0 -fstrength-reduce -fomit-frame-pointer -finline-functions \
		-ffreestanding -nostdlib -nostartfiles -nodefaultlibs -fno-builtin \
		-falign-functions -falign-loops -falign-jumps -falign-labels

NFLAGS = -g

LDFLAGS = -g

.PHONY: all run debug format clean init kernel16

all: init build/bootloader.bin build/kernel.bin kernel16
	@if [ $$(stat -c '%s' kernel16/build/kernel16.bin) -gt 51200 ]; then \
		echo "Kernel16 code exceeds 51200 bytes." >&2; \
		echo "Please check layout.txt file." >&2; \
		exit 1; \
	fi
	dd if=/dev/zero of=build/ghostos.bin bs=1M count=1
	mkfs.fat -F12 -R101 build/ghostos.bin
	dd if=build/bootloader.bin of=build/ghostos.bin bs=1 seek=62 count=448 conv=notrunc
	dd if=kernel16/build/kernel16.bin of=build/ghostos.bin bs=512 seek=1 conv=notrunc

run: all
	qemu-system-i386 -hda build/ghostos.bin

debug: all
	gdb \
		-ex "set confirm off" \
		-ex "add-symbol-file build/kernel.tmp.o 0x100000" \
		-ex "break _start" \
		-ex "target remote | qemu-system-i386 -hda build/ghostos.bin -S -gdb stdio" \
		-ex "continue"

format:
	clang-format-18 -i *.h *.c
	$(MAKE) -C kernel16 format

clean:
	rm -rf build
	$(MAKE) -C kernel16 clean

init:
	mkdir -p build

# Order of the files is important
build/kernel.bin: build/kernel.asm.o build/kernel.o build/gdt.o build/gdt.asm.o build/idt.o build/idt.asm.o build/hal.o build/memory.o build/string.o build/heap.o build/paging.o build/paging.asm.o build/tss.o build/tss.asm.o build/disk.o build/keyboard.o build/console.o build/task.o build/task.asm.o build/process.o
	i686-elf-ld -relocatable $(LDFLAGS) build/kernel.asm.o build/kernel.o build/gdt.o build/gdt.asm.o build/idt.o build/idt.asm.o build/hal.o build/memory.o build/string.o build/heap.o build/paging.o build/paging.asm.o build/tss.o build/tss.asm.o build/disk.o build/keyboard.o build/console.o build/task.o build/task.asm.o build/process.o -o build/kernel.tmp.o
	i686-elf-gcc $(FLAGS) -T linker.ld build/kernel.tmp.o -o build/kernel.bin -static-libgcc -lgcc

build/kernel.o: kernel.c constants.h
	i686-elf-gcc $(FLAGS) -c kernel.c -o build/kernel.o

build/kernel.asm.o: kernel.asm constants.inc
	nasm -f elf $(NFLAGS) kernel.asm -o build/kernel.asm.o

build/gdt.o: gdt.c constants.h
	i686-elf-gcc $(FLAGS) -c gdt.c -o build/gdt.o

build/gdt.asm.o: gdt.asm constants.inc
	nasm -f elf $(NFLAGS) gdt.asm -o build/gdt.asm.o

build/idt.o: idt.c constants.h
	i686-elf-gcc $(FLAGS) -c idt.c -o build/idt.o

build/idt.asm.o: idt.asm constants.inc
	nasm -f elf $(NFLAGS) idt.asm -o build/idt.asm.o

build/hal.o: hal.c constants.h
	i686-elf-gcc $(FLAGS) -c hal.c -o build/hal.o

build/memory.o: memory.c constants.h
	i686-elf-gcc $(FLAGS) -c memory.c -o build/memory.o

build/string.o: string.c constants.h
	i686-elf-gcc $(FLAGS) -c string.c -o build/string.o

build/heap.o: heap.c constants.h
	i686-elf-gcc $(FLAGS) -c heap.c -o build/heap.o

build/paging.o: paging.c constants.h
	i686-elf-gcc $(FLAGS) -c paging.c -o build/paging.o

build/paging.asm.o: paging.asm constants.inc
	nasm -f elf $(NFLAGS) paging.asm -o build/paging.asm.o

build/tss.o: tss.c constants.h
	i686-elf-gcc $(FLAGS) -c tss.c -o build/tss.o

build/tss.asm.o: tss.asm constants.inc
	nasm -f elf $(NFLAGS) tss.asm -o build/tss.asm.o

build/disk.o: disk.c constants.h
	i686-elf-gcc $(FLAGS) -c disk.c -o build/disk.o

build/keyboard.o: keyboard.c constants.h
	i686-elf-gcc $(FLAGS) -c keyboard.c -o build/keyboard.o

build/console.o: console.c constants.h
	i686-elf-gcc $(FLAGS) -c console.c -o build/console.o

build/task.o: task.c constants.h
	i686-elf-gcc $(FLAGS) -c task.c -o build/task.o

build/task.asm.o: task.asm constants.inc
	nasm -f elf $(NFLAGS) task.asm -o build/task.asm.o

build/process.o: process.c constants.h
	i686-elf-gcc $(FLAGS) -c process.c -o build/process.o

build/bootloader.bin: bootloader.asm constants.inc
	nasm bootloader.asm -o build/bootloader.bin

constants.h: constants.def
	sed -e '/^#/d' -e '/^[[:space:]]*$$/d' -e 's/^/#define /' constants.def > constants.h

constants.inc: constants.def
	sed -e '/^#/d' -e '/^[[:space:]]*$$/d' -e 's/^/%define /' constants.def > constants.inc

kernel16: constants.h constants.inc
	$(MAKE) -C kernel16
