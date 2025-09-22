use std::ops::{Deref, DerefMut, Index, IndexMut};

use crate::Word;
use crate::memory::MainMemory;
use crate::opcode::JCmovFun;

#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error("invalid register {0:#x}")]
  InvalidRegister(u8),
}

type RegisterSlot = Word;

#[derive(Debug)]
struct Registers([RegisterSlot; 15]);

impl Registers {
  fn new() -> Self {
    let mut regs = [0; 15];
    // Initialize stack pointer to top of memory (grows down)
    regs[Register::Rsp as usize] = MainMemory::MEMORY_SIZE as i64 - 8;
    Self(regs)
  }
}

impl Deref for Registers {
  type Target = [RegisterSlot; 15];

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for Registers {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

#[derive(Debug)]
struct Flags {
  zf: bool,
  sf: bool,
  of: bool,
}

impl Flags {
  fn new() -> Self {
    Self {
      zf: false,
      sf: false,
      of: false,
    }
  }

  fn eval_condition(&self, cond: &JCmovFun) -> bool {
    match cond {
      // SF^OF | ZF
      JCmovFun::LessEqual => (self.sf ^ self.of) | self.zf,
      // SF^OF
      JCmovFun::Less => self.sf ^ self.of,
      // ZF
      JCmovFun::Equal => self.zf,
      // !ZF
      JCmovFun::NotEqual => !self.zf,
      // !(SF^OF)
      JCmovFun::GreaterEqual => !(self.sf ^ self.of),
      // !(SF^OF) & !ZF
      JCmovFun::Greater => !(self.sf ^ self.of) & !self.zf,
    }
  }
}

#[derive(Debug)]
pub(crate) struct RegisterFile {
  registers: Registers,
  flags: Flags,
}

impl RegisterFile {
  pub(crate) fn eval_condition(&self, cond: &JCmovFun) -> bool {
    self.flags.eval_condition(cond)
  }
}

impl Default for RegisterFile {
  fn default() -> Self {
    Self {
      registers: Registers::new(),
      flags: Flags::new(),
    }
  }
}

#[derive(Clone, Copy)]
pub(crate) enum Register {
  Rax = 0,
  Rcx = 1,
  Rdx = 2,
  Rbx = 3,
  Rsp = 4,
  Rbp = 5,
  Rsi = 6,
  Rdi = 7,
  R8 = 8,
  R9 = 9,
  R10 = 10,
  R11 = 11,
  R12 = 12,
  R13 = 13,
  R14 = 14,
}

impl TryFrom<u8> for Register {
  type Error = Error;

  fn try_from(byte: u8) -> Result<Self, Self::Error> {
    let reg = match byte {
      0x0 => Register::Rax,
      0x1 => Register::Rcx,
      0x2 => Register::Rdx,
      0x3 => Register::Rbx,
      0x4 => Register::Rsp,
      0x5 => Register::Rbp,
      0x6 => Register::Rsi,
      0x7 => Register::Rdi,
      0x8 => Register::R8,
      0x9 => Register::R9,
      0xa => Register::R10,
      0xb => Register::R11,
      0xc => Register::R12,
      0xd => Register::R13,
      0xe => Register::R14,
      _ => return Err(Error::InvalidRegister(byte)),
    };
    Ok(reg)
  }
}

impl Index<Register> for RegisterFile {
  type Output = RegisterSlot;

  fn index(&self, index: Register) -> &Self::Output {
    &self.registers[index as usize]
  }
}

impl IndexMut<Register> for RegisterFile {
  fn index_mut(&mut self, index: Register) -> &mut Self::Output {
    &mut self.registers[index as usize]
  }
}

pub(crate) enum Flag {
  ZF, // zero flag
  SF, // sign flag
  OF, // overflow flag
}

impl Index<Flag> for RegisterFile {
  type Output = bool;

  fn index(&self, index: Flag) -> &Self::Output {
    match index {
      Flag::ZF => &self.flags.zf,
      Flag::SF => &self.flags.sf,
      Flag::OF => &self.flags.of,
    }
  }
}

impl IndexMut<Flag> for RegisterFile {
  fn index_mut(&mut self, index: Flag) -> &mut Self::Output {
    match index {
      Flag::ZF => &mut self.flags.zf,
      Flag::SF => &mut self.flags.sf,
      Flag::OF => &mut self.flags.of,
    }
  }
}
