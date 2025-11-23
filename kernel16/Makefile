TARGET_EXEC := kernel16.bin

BUILD_DIR := ./build
SRC_DIRS := ./

CFLAGS := -ms -zl -s -wx

SRCS := $(shell find $(SRC_DIRS) -name '*.c' -or -name '*.asm')
OBJS := $(SRCS:%=$(BUILD_DIR)/%.obj)
DEPS := $(OBJS:.obj=.d)

INC_DIRS := $(shell find $(SRC_DIRS) -type d)
INC_FLAGS := $(addprefix -i=,$(INC_DIRS))

$(BUILD_DIR)/$(TARGET_EXEC): $(OBJS) linker.lnk
	/opt/watcom/binl/wlink NAME $@ FILE \{ $(OBJS) \} OPTION MAP=$(BUILD_DIR)/kernel16.map @linker.lnk

$(BUILD_DIR)/%.c.obj: %.c
	mkdir -p $(dir $@)
	/opt/watcom/binl/wcc $(INC_FLAGS) -ad=$(@:.obj=.d) $(CFLAGS) $< -fo=$@

$(BUILD_DIR)/%.asm.obj: %.asm ../constants.inc
	mkdir -p $(dir $@)
	nasm -f obj $< -o $@

.PHONY: format
format:
	clang-format -i *.h *.c

.PHONY: clean
clean:
	rm -r $(BUILD_DIR)

-include $(DEPS)
