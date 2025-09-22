use std::fmt;

use crate::{BLOCK_SIZE, Block};

#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error("invalid memory accessed at address {0:#x}")]
  InvalidAddress(usize),

  #[error("unaligned memory access at address {0:#x}")]
  UnalignedAccess(usize),
}

pub(crate) struct MainMemory {
  bytes: Vec<u8>,
}

impl MainMemory {
  pub(crate) const MEMORY_SIZE: usize = 1 << 16; // 64KB of memory

  pub(crate) fn read(&self, addr: usize) -> Result<Block, Error> {
    if addr % BLOCK_SIZE != 0 {
      return Err(Error::UnalignedAccess(addr));
    }
    if addr + BLOCK_SIZE > self.bytes.len() {
      return Err(Error::InvalidAddress(addr));
    }
    // safety:
    // - we verified 8 byte alignment (can use read)
    // - we made sure we are reading within valid bytes
    let value = unsafe {
      let block = self.bytes.as_ptr().add(addr) as *const Block;
      block.read()
    };
    Ok(value)
  }

  pub(crate) fn write(&mut self, addr: usize, value: Block) -> Result<(), Error> {
    if addr % BLOCK_SIZE != 0 {
      return Err(Error::UnalignedAccess(addr));
    }
    if addr + BLOCK_SIZE > self.bytes.len() {
      return Err(Error::InvalidAddress(addr));
    }
    // safety:
    // - we verified 8 byte alignment (can use write)
    // - we made sure we are writing within valid bytes
    unsafe {
      let block = self.bytes.as_ptr().add(addr) as *mut Block;
      block.write(value);
    }
    Ok(())
  }
}

impl Default for MainMemory {
  fn default() -> Self {
    Self {
      bytes: vec![0; Self::MEMORY_SIZE],
    }
  }
}

impl fmt::Debug for MainMemory {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "MainMemory {{ size: {} bytes }}", self.bytes.len())
  }
}
