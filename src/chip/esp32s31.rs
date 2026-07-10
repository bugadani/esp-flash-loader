use crate::{
    efuse::{read_field, EfuseInfo},
    flash::MemSpi,
    rom::{RomDataTable, RomDataTables},
};

// Max of 256MB
pub const MAX_FLASH_SIZE: u32 = 0x10000000;

// No ROM data init tables needed; ROM boot already initializes its own data.
pub const ROM_DATA_TABLES: RomDataTables = &[];

// Suppress dead-code warning: ROM_DATA_TABLES is an empty slice but the type
// still requires the element type to be in scope.
const _: RomDataTable = RomDataTable {
    min_revision: 0,
    data_start: 0,
    data_end: 0,
    bss_start: 0,
    bss_end: 0,
};

pub const ROM_TABLE_ENTRY_SIZE: u32 = 12;

pub const EFUSE_INFO: EfuseInfo = EfuseInfo {
    // EFUSE peripheral base = 0x20715000; readable data starts at offset 0x2C
    block0: 0x2071_5000 + 0x2C,
    block_sizes: &[6, 6, 8, 8, 8, 8, 8, 8, 8, 8, 8],
};

pub const MEM_SPI: MemSpi = MemSpi {
    // SPIMEM1 (SPI1 flash controller) base address
    base: 0x2050_1000,
    cmd: 0x00,
    addr: 0x04,
    ctrl: 0x08,
    user: 0x18,
    user1: 0x1C,
    user2: 0x20,
    miso_dlen: 0x28,
    data_buf_0: 0x58,
};

pub struct CpuSaveState {}

impl CpuSaveState {
    pub const fn new() -> Self {
        CpuSaveState {}
    }

    pub fn set_max_cpu_clock(&mut self) {}

    pub fn restore(&self) {}
}

// EFUSE_BLK1, bit 118, 2 bits (WAFER_VERSION_MAJOR)
pub fn major_chip_version() -> u8 {
    read_field::<1, 118, 2>()
}

// EFUSE_BLK1, bit 114, 4 bits (WAFER_VERSION_MINOR)
pub fn minor_chip_version() -> u8 {
    read_field::<1, 114, 4>()
}

/// Ensures that data (e.g. constants) are accessed through the data bus.
pub unsafe fn read_via_data_bus(s: &u8) -> u8 {
    unsafe { core::ptr::read(s as *const u8) }
}
