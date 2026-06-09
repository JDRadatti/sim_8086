use std::error::Error;
use std::fmt;

pub struct Memory {
    buffer: Vec<u8>,
}

pub struct Region {
    pub address: usize,
    pub size: usize,
}

impl Memory {
    pub fn new(byte_count: usize) -> Memory {
        Memory {
            buffer: vec![0; byte_count],
        }
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn read(&self, region: &Region) -> Result<&[u8], MemoryError> {
        self.check_bounds(region)?;
        Ok(&self.buffer[region.address..][..region.size])
    }

    pub fn write(&mut self, data: &[u8], region: &Region) -> Result<(), MemoryError> {
        self.check_bounds(region)?;
        if data.len() != region.size {
            return Err(MemoryError::SegmentationFault);
        }
        self.buffer[region.address..][..region.size].copy_from_slice(data);
        Ok(())
    }

    pub fn memcopy(&mut self, source: &Region, destination: &Region) -> Result<(), MemoryError> {
        self.check_bounds(source)?;
        self.check_bounds(destination)?;
        let data = self.read(source)?.to_vec();
        self.write(&data, destination)
    }

    fn check_bounds(&self, region: &Region) -> Result<(), MemoryError> {
        region
            .address
            .checked_add(region.size)
            .filter(|&end| end <= self.buffer.len())
            .ok_or(MemoryError::SegmentationFault)?;
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub enum MemoryError {
    SegmentationFault,
}

impl fmt::Display for MemoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MemoryError::SegmentationFault => write!(f, "segmentation fault"),
        }
    }
}

impl Error for MemoryError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_returns_correct_slice() {
        let mut mem = Memory::new(16);
        mem.buffer[4..8].copy_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD]);
        let result = mem
            .read(&Region {
                address: 4,
                size: 4,
            })
            .unwrap();
        assert_eq!(result, &[0xAA, 0xBB, 0xCC, 0xDD]);
    }

    #[test]
    fn read_zero_size_at_valid_address() {
        let mem = Memory::new(16);
        let result = mem
            .read(&Region {
                address: 8,
                size: 0,
            })
            .unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn read_out_of_bounds_returns_error() {
        let mem = Memory::new(16);
        let result = mem.read(&Region {
            address: 8,
            size: 16,
        });
        assert_eq!(result, Err(MemoryError::SegmentationFault));
    }

    #[test]
    fn read_address_past_end_returns_error() {
        let mem = Memory::new(16);
        let result = mem.read(&Region {
            address: 16,
            size: 1,
        });
        assert_eq!(result, Err(MemoryError::SegmentationFault));
    }

    #[test]
    fn read_overflow_wraps_returns_error() {
        let mem = Memory::new(1024);
        let result = mem.read(&Region {
            address: usize::MAX,
            size: 1,
        });
        assert_eq!(result, Err(MemoryError::SegmentationFault));
    }

    #[test]
    fn write_writes_data_correctly() {
        let mut mem = Memory::new(16);
        mem.write(
            &[0x11, 0x22, 0x33],
            &Region {
                address: 2,
                size: 3,
            },
        )
        .unwrap();
        assert_eq!(&mem.buffer[2..5], &[0x11, 0x22, 0x33]);
        assert_eq!(&mem.buffer[0..2], &[0, 0]);
        assert_eq!(&mem.buffer[5..16], &[0; 11]);
    }

    #[test]
    fn write_at_start_of_buffer() {
        let mut mem = Memory::new(4);
        mem.write(
            &[0xDE, 0xAD],
            &Region {
                address: 0,
                size: 2,
            },
        )
        .unwrap();
        assert_eq!(&mem.buffer[0..2], &[0xDE, 0xAD]);
    }

    #[test]
    fn write_at_end_of_buffer() {
        let mut mem = Memory::new(4);
        mem.write(
            &[0xFF],
            &Region {
                address: 3,
                size: 1,
            },
        )
        .unwrap();
        assert_eq!(mem.buffer[3], 0xFF);
    }

    #[test]
    fn write_data_size_mismatch_returns_error() {
        let mut mem = Memory::new(16);
        let result = mem.write(
            &[1, 2, 3],
            &Region {
                address: 0,
                size: 4,
            },
        );
        assert_eq!(result, Err(MemoryError::SegmentationFault));
    }

    #[test]
    fn write_out_of_bounds_returns_error() {
        let mut mem = Memory::new(4);
        let result = mem.write(
            &[1, 2],
            &Region {
                address: 3,
                size: 2,
            },
        );
        assert_eq!(result, Err(MemoryError::SegmentationFault));
    }

    #[test]
    fn write_address_past_end_returns_error() {
        let mut mem = Memory::new(4);
        let result = mem.write(
            &[1],
            &Region {
                address: 4,
                size: 1,
            },
        );
        assert_eq!(result, Err(MemoryError::SegmentationFault));
    }

    #[test]
    fn write_does_not_corrupt_on_error() {
        let mut mem = Memory::new(4);
        mem.buffer.copy_from_slice(&[0; 4]);
        let _ = mem.write(
            &[1, 2, 3],
            &Region {
                address: 0,
                size: 4,
            },
        );
        assert_eq!(&mem.buffer, &[0, 0, 0, 0]);
    }

    #[test]
    fn memcopy_copies_data_correctly() {
        let mut mem = Memory::new(16);
        mem.write(
            &[0xAA, 0xBB, 0xCC],
            &Region {
                address: 0,
                size: 3,
            },
        )
        .unwrap();
        mem.memcopy(
            &Region {
                address: 0,
                size: 3,
            },
            &Region {
                address: 8,
                size: 3,
            },
        )
        .unwrap();
        assert_eq!(&mem.buffer[8..11], &[0xAA, 0xBB, 0xCC]);
    }

    #[test]
    fn memcopy_with_overlapping_regions_works() {
        let mut mem = Memory::new(16);
        mem.write(
            &[1, 2, 3, 4, 5],
            &Region {
                address: 0,
                size: 5,
            },
        )
        .unwrap();
        mem.memcopy(
            &Region {
                address: 0,
                size: 5,
            },
            &Region {
                address: 2,
                size: 5,
            },
        )
        .unwrap();
        assert_eq!(&mem.buffer[2..7], &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn memcopy_source_out_of_bounds_returns_error() {
        let mut mem = Memory::new(4);
        let result = mem.memcopy(
            &Region {
                address: 2,
                size: 4,
            },
            &Region {
                address: 0,
                size: 2,
            },
        );
        assert_eq!(result, Err(MemoryError::SegmentationFault));
    }

    #[test]
    fn memcopy_destination_out_of_bounds_returns_error() {
        let mut mem = Memory::new(4);
        let result = mem.memcopy(
            &Region {
                address: 0,
                size: 2,
            },
            &Region {
                address: 2,
                size: 4,
            },
        );
        assert_eq!(result, Err(MemoryError::SegmentationFault));
    }

    #[test]
    fn multiple_writes_accumulate() {
        let mut mem = Memory::new(8);
        mem.write(
            &[0x0F],
            &Region {
                address: 0,
                size: 1,
            },
        )
        .unwrap();
        mem.write(
            &[0xF0],
            &Region {
                address: 1,
                size: 1,
            },
        )
        .unwrap();
        mem.write(
            &[0xFF],
            &Region {
                address: 2,
                size: 1,
            },
        )
        .unwrap();
        assert_eq!(&mem.buffer[0..3], &[0x0F, 0xF0, 0xFF]);
    }

    #[test]
    fn read_after_write_returns_written_data() {
        let mut mem = Memory::new(4);
        mem.write(
            &[0xAB, 0xCD],
            &Region {
                address: 1,
                size: 2,
            },
        )
        .unwrap();
        let result = mem
            .read(&Region {
                address: 1,
                size: 2,
            })
            .unwrap();
        assert_eq!(result, &[0xAB, 0xCD]);
    }
}
