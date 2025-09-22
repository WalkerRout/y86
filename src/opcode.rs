#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error("invalid opcode {0}")]
  InvalidOpcode(u8),
}

#[derive(Debug)]
pub(crate) enum OpFun {
  Add,
  Sub,
  And,
  Xor,
  Mul,
  Div,
  Mod,
}

impl TryFrom<u8> for OpFun {
  type Error = Error;

  fn try_from(byte: u8) -> Result<Self, Self::Error> {
    let op = match byte {
      0x0 => OpFun::Add,
      0x1 => OpFun::Sub,
      0x2 => OpFun::And,
      0x3 => OpFun::Xor,
      0x4 => OpFun::Mul,
      0x5 => OpFun::Div,
      0x6 => OpFun::Mod,
      _ => return Err(Error::InvalidOpcode(byte)),
    };
    Ok(op)
  }
}

#[derive(Debug)]
pub(crate) enum JCmovFun {
  LessEqual,    // le (ifun = 1)
  Less,         // l (ifun = 2)
  Equal,        // e (ifun = 3)
  NotEqual,     // ne (ifun = 4)
  GreaterEqual, // ge (ifun = 5)
  Greater,      // g (ifun = 6)
}

impl TryFrom<u8> for JCmovFun {
  type Error = Error;

  fn try_from(byte: u8) -> Result<Self, Self::Error> {
    let op = match byte {
      0x1 => JCmovFun::LessEqual,
      0x2 => JCmovFun::Less,
      0x3 => JCmovFun::Equal,
      0x4 => JCmovFun::NotEqual,
      0x5 => JCmovFun::GreaterEqual,
      0x6 => JCmovFun::Greater,
      _ => return Err(Error::InvalidOpcode(byte)),
    };
    Ok(op)
  }
}

#[derive(Debug)]
pub(crate) enum Opcode {
  Halt,
  Nop,
  Rrmovq,
  Cmovxx(JCmovFun),
  Irmovq,
  Rmmovq,
  Mrmovq,
  Opq(OpFun),
  Jxx(JCmovFun),
  Call,
  Ret,
  Pushq,
  Popq,
}

impl TryFrom<u8> for Opcode {
  type Error = Error;

  fn try_from(byte: u8) -> Result<Self, Self::Error> {
    let (high, low) = (byte >> 4, byte & 0xf);
    let op = match high {
      0x0 => Opcode::Halt,
      0x1 => Opcode::Nop,
      0x2 => match low {
        0x0 => Opcode::Rrmovq,
        _ => Opcode::Cmovxx(JCmovFun::try_from(low)?),
      },
      0x3 => Opcode::Irmovq,
      0x4 => Opcode::Rmmovq,
      0x5 => Opcode::Mrmovq,
      0x6 => Opcode::Opq(OpFun::try_from(low)?),
      0x7 => Opcode::Jxx(JCmovFun::try_from(low)?),
      0x8 => Opcode::Call,
      0x9 => Opcode::Ret,
      0xA => Opcode::Pushq,
      0xB => Opcode::Popq,
      _ => return Err(Error::InvalidOpcode(byte)),
    };
    Ok(op)
  }
}
