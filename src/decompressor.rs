//! Decompresses the image, then writes or verifies it.

use crate::{
    flash,
    tinfl::{OutBuffer, TinflDecompressor, TINFL_STATUS_DONE, TINFL_STATUS_NEEDS_MORE_INPUT},
    ERROR_BASE_FLASH, ERROR_BASE_INTERNAL, ERROR_BASE_TINFL,
};

pub struct Decompressor {
    decompressor: TinflDecompressor,
    output: OutBuffer,
    image_start: Option<u32>,
    offset: u32,
    remaining_compressed: usize,
}

impl Decompressor {
    pub const fn new() -> Self {
        Self {
            image_start: None,
            offset: 0,
            output: OutBuffer::new(),
            remaining_compressed: 0,
            decompressor: TinflDecompressor::new(),
        }
    }

    fn reinit(&mut self, address: u32, compressed: u32) {
        self.image_start = Some(address);
        self.offset = 0;

        self.remaining_compressed = compressed as usize;

        self.decompressor = TinflDecompressor::new();
        self.output.take(|_| {});
    }

    fn decompress(&mut self, input: &[u8], process: fn(u32, &[u8]) -> i32) -> i32 {
        if self.remaining_compressed == 0 {
            return ERROR_BASE_INTERNAL - 3;
        }

        // We may have to cut off some padding bytes.
        let chunk_len = self.remaining_compressed.min(input.len());
        self.remaining_compressed -= chunk_len;

        // Signal tinfl_decompress() that this is the last chunk
        let last = self.remaining_compressed == 0;

        // Iterate through all the input
        let mut input = &input[..chunk_len];
        let mut status = TINFL_STATUS_NEEDS_MORE_INPUT as i32;
        while !input.is_empty() && status > TINFL_STATUS_DONE as i32 {
            status = self
                .decompressor
                .decompress(&mut input, &mut self.output, last);

            if status == TINFL_STATUS_DONE as i32 || self.output.full() {
                // We're either finished or the decompressor can't continue
                // until we flush the buffer.
                let flush_status = self.flush(process);

                if flush_status < 0 {
                    return ERROR_BASE_FLASH + flush_status;
                }
            }
        }

        if status < TINFL_STATUS_DONE as i32 {
            ERROR_BASE_TINFL + status
        } else {
            0
        }
    }

    pub fn flush(&mut self, process: fn(address: u32, data: &[u8]) -> i32) -> i32 {
        let mut offset = self.offset;
        let address = self.image_start.unwrap_or(0) + offset;

        // Take buffer contents, write to flash and update offset.
        let status = self.output.take(|data| {
            offset += data.len() as u32;

            process(address, data)
        });

        self.offset = offset;

        status
    }

    fn handle_compressed(
        &mut self,
        address: u32,
        mut data: &[u8],
        process: fn(u32, &[u8]) -> i32,
    ) -> i32 {
        if self.image_start != Some(address) {
            if data.len() < 4 {
                // We don't have enough bytes to read the length
                return ERROR_BASE_INTERNAL - 4;
            }

            // Image length is prepended to the first chunk, cut it off.
            let (length_bytes, remaining) = data.split_at(4);
            data = remaining;

            let compressed_length = u32::from_le_bytes([
                length_bytes[0],
                length_bytes[1],
                length_bytes[2],
                length_bytes[3],
            ]);

            self.reinit(address, compressed_length);
        }
        self.decompress(data, process)
    }

    pub fn program(&mut self, address: u32, data: &[u8]) -> i32 {
        self.handle_compressed(address, data, write_to_flash)
    }

    pub fn verify(&mut self, address: u32, data: &[u8]) -> i32 {
        // We're supposed to return the address up to which we've verified.
        // However, we process compressed data and the caller expects us to respond in terms of
        // compressed offsets, so we don't actually know where comparison fails.
        let status = if self.handle_compressed(address, data, verify_flash) == 0 {
            address + data.len() as u32
        } else {
            address
        };

        status as i32
    }
}

fn write_to_flash(address: u32, data: &[u8]) -> i32 {
    let status = flash::write_flash(address, data);

    if status < 0 {
        return ERROR_BASE_FLASH + status;
    }

    0
}

fn verify_flash(mut address: u32, mut data: &[u8]) -> i32 {
    const READBACK_BUFFER: usize = 256;
    let mut readback = unsafe {
        let mut buf = core::mem::MaybeUninit::<[u8; READBACK_BUFFER]>::uninit();
        for i in 0..READBACK_BUFFER {
            buf.as_mut_ptr().cast::<u8>().write_volatile(i as u8);
        }
        buf.assume_init()
    };

    while !data.is_empty() {
        let chunk_size = READBACK_BUFFER.min(data.len());
        let (slice, rest) = unsafe {
            // SAFETY: skip is always at most `data.len()`
            data.split_at_unchecked(chunk_size)
        };
        data = rest;

        let readback_slice = &mut readback[..chunk_size];

        let status = flash::read_flash(address, readback_slice);
        if status < 0 {
            return -1;
        }

        for (a, b) in slice.iter().zip(readback_slice.iter()) {
            if a != b {
                return -1;
            }
        }

        address += chunk_size as u32;
    }

    0
}
