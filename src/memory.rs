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
