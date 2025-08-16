#include "disk.h"

#include "hal.h"

void
ReadSector(uint32_t lba, uint8_t* buffer)
{
    PortOutByte(0x1f6, 0xe0 | ((lba >> 24) & 0x0f));
    PortOutByte(0x1f2, 1);
    PortOutByte(0x1f3, lba & 0xff);
    PortOutByte(0x1f4, (lba >> 8) & 0xff);
    PortOutByte(0x1f5, (lba >> 16) & 0xff);
    PortOutByte(0x1f7, 0x20);

    while ((PortInByte(0x1f7) & 0x08) == 0)
        ;

    uint16_t* buffer16 = (uint16_t*)buffer;
    for (int i = 0; i < 256; i++) {
        buffer16[i] = PortInWord(0x1f0);
    }
}
