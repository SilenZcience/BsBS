/*
 * Contains glue functions for the gb struct
 *
 * Author: Fabian Ruhland, Heinrich Heine University Duesseldorf, 2026-04-01
 * License: GPLv3
 */

 #include <stdint.h>

#include "peanut-gb.h"

int gb_size() {
    return sizeof(struct gb_s);
}

uint8_t* gb_get_joypad_ptr(struct gb_s* gb) {
    return &gb->direct.joypad;
}

struct gb_cart_debug_info {
    uint8_t enable_cart_ram;
    uint8_t cart_ram;
    uint8_t mbc;
    uint8_t cart_ram_bank;
    uint8_t cart_mode_select;
    uint32_t num_ram_banks;
    uint32_t num_rom_banks_mask;
    uint32_t selected_rom_bank;
};

void gb_debug_cart_info(struct gb_s* gb, struct gb_cart_debug_info* info) {
    info->enable_cart_ram = gb->enable_cart_ram;
    info->cart_ram = gb->cart_ram;
    info->mbc = gb->mbc;
    info->cart_ram_bank = gb->cart_ram_bank;
    info->cart_mode_select = gb->cart_mode_select;
    info->num_ram_banks = gb->num_ram_banks;
    info->num_rom_banks_mask = gb->num_rom_banks_mask;
    info->selected_rom_bank = gb->selected_rom_bank;
}