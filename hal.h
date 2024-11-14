#pragma once

#include "types.h"

uint8_t PortInByte(uint16_t port);
uint16_t PortInWord(uint16_t port);
uint32_t PortInDword(uint16_t port);

void PortOutByte(uint16_t port, uint8_t v);
void PortOutWord(uint16_t port, uint16_t v);
void PortOutDword(uint16_t port, uint32_t v);
