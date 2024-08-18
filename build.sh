#!/bin/bash

set -e

if [ "$1" == "build" ]; then
    nasm stage1.asm -o stage1.bin
    nasm stage2.asm -o stage2.bin
    clang src/*.c kernel.c -o kernel.bin -nostdlib -nodefaultlibs -fno-exceptions -fno-asynchronous-unwind-tables -Wl,--build-id=none -Wl,--no-dynamic-linker -Wl,-z,norelro -Weverything -Wno-declaration-after-statement -Wno-unsafe-buffer-usage
    objcopy --remove-section=.gnu.hash --remove-section=.dynsym --remove-section=.dynstr --remove-section=.eh_frame --remove-section=.dynamic --remove-section=.comment kernel.bin
    strip kernel.bin
    if [ $(stat -c %s kernel.bin) -gt 16384 ]; then
        echo "Kernel code exceeds 16384 bytes." >&2
        exit 1
    fi
    cat stage1.bin stage2.bin kernel.bin > os.bin
    exit 0
fi

if [ "$1" == "run" ]; then
    bochs -f bochsrc
    exit 0
fi

if [ "$1" == "clean" ]; then
    rm *.bin
    rm *.log
    exit 0
fi
