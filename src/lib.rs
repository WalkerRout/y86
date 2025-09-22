use std::mem;

pub mod memory;
pub mod opcode;
pub mod region;
pub mod register;
pub mod vm;

pub(crate) type Word = i64;

pub(crate) type Block = Word;

pub(crate) const BLOCK_SIZE: usize = mem::size_of::<Block>();
