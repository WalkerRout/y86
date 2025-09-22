use crate::Block;
use crate::memory::{self, MainMemory};
use crate::opcode::{self, JCmovFun, OpFun, Opcode};
use crate::region::Region;
use crate::register::{self, Flag, Register, RegisterFile};

#[derive(Debug, Clone, Copy, PartialEq)]
enum State {
  Active,
  Halted,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
  #[error("machine is halted")]
  MachineHalted,

  #[error("reached the end of instructions at ip {0}")]
  EndOfInstructions(usize),

  #[error("division by zero")]
  DivisionByZero,

  #[error("opcode error - {0}")]
  OpcodeError(#[from] opcode::Error),

  #[error("memory error - {0}")]
  MemoryError(#[from] memory::Error),

  #[error("register error - {0}")]
  RegisterError(#[from] register::Error),
}

#[derive(Debug)]
pub struct Vm {
  ip: usize,
  memory: MainMemory,
  reg_file: RegisterFile,
  state: State,
}

impl Vm {
  pub fn new() -> Self {
    Self {
      ip: 0,
      memory: MainMemory::default(),
      reg_file: RegisterFile::default(),
      state: State::Active,
    }
  }

  pub fn step<R>(&mut self, region: &R) -> Result<(), Error>
  where
    R: Region,
  {
    if self.state == State::Halted {
      return Err(Error::MachineHalted);
    }
    let mut task = Task::new(self, region);
    task.run()
  }

  fn read_block(&self, address: usize) -> Result<Block, Error> {
    Ok(self.memory.read(address)?)
  }

  fn write_block(&mut self, address: usize, value: Block) -> Result<(), Error> {
    Ok(self.memory.write(address, value)?)
  }
}

impl Default for Vm {
  fn default() -> Self {
    Self::new()
  }
}

struct Task<'vm, 'region, R> {
  vm: &'vm mut Vm,
  region: &'region R,
}

impl<'vm, 'region, R> Task<'vm, 'region, R>
where
  R: Region,
{
  fn new(vm: &'vm mut Vm, region: &'region R) -> Self {
    Self { vm, region }
  }

  fn eat(&mut self) -> Result<u8, Error> {
    self
      .region
      .instructions()
      .get(self.vm.ip)
      .map(|b| {
        self.vm.ip += 1;
        *b
      })
      .ok_or(Error::EndOfInstructions(self.vm.ip))
  }

  fn eat_immediate(&mut self) -> Result<Block, Error> {
    let mut bytes = [0u8; 8];
    for byte in &mut bytes {
      *byte = self.eat()?;
    }
    // le convert
    Ok(Block::from_le_bytes(bytes))
  }

  fn run(&mut self) -> Result<(), Error> {
    let opcode = Opcode::try_from(self.eat()?)?;
    match opcode {
      Opcode::Halt => halt(self)?,
      Opcode::Nop => nop(self)?,
      Opcode::Rrmovq => rrmovq(self)?,
      Opcode::Cmovxx(cond) => cmovxx(self, cond)?,
      Opcode::Irmovq => irmovq(self)?,
      Opcode::Rmmovq => rmmovq(self)?,
      Opcode::Mrmovq => mrmovq(self)?,
      Opcode::Opq(fun) => opq(self, fun)?,
      Opcode::Jxx(cond) => jxx(self, cond)?,
      Opcode::Call => call(self)?,
      Opcode::Ret => ret(self)?,
      Opcode::Pushq => pushq(self)?,
      Opcode::Popq => popq(self)?,
    }
    Ok(())
  }
}

fn halt(task: &mut Task<'_, '_, impl Region>) -> Result<(), Error> {
  task.vm.state = State::Halted;
  Ok(())
}

fn nop(_task: &mut Task<'_, '_, impl Region>) -> Result<(), Error> {
  Ok(())
}

fn rrmovq(task: &mut Task<'_, '_, impl Region>) -> Result<(), Error> {
  let byte = task.eat()?;
  let ra = Register::try_from(byte >> 4)?; // src
  let rb = Register::try_from(byte & 0xf)?; // dest
  let val_a = task.vm.reg_file[ra];
  task.vm.reg_file[rb] = val_a;
  Ok(())
}

fn cmovxx(task: &mut Task<'_, '_, impl Region>, cond: JCmovFun) -> Result<(), Error> {
  let byte = task.eat()?;
  let ra = Register::try_from(byte >> 4)?; // src
  let rb = Register::try_from(byte & 0xf)?; // dest
  // only move if condition is met
  if task.vm.reg_file.eval_condition(&cond) {
    let val_a = task.vm.reg_file[ra];
    task.vm.reg_file[rb] = val_a;
  }
  Ok(())
}

fn irmovq(task: &mut Task<'_, '_, impl Region>) -> Result<(), Error> {
  let byte = task.eat()?;
  let rb = Register::try_from(byte & 0xf)?; // dest
  let val_c = task.eat_immediate()?;
  task.vm.reg_file[rb] = val_c;
  Ok(())
}

fn rmmovq(task: &mut Task<'_, '_, impl Region>) -> Result<(), Error> {
  let byte = task.eat()?;
  let ra = Register::try_from(byte >> 4)?; // src
  let rb = Register::try_from(byte & 0xf)?; // base
  let val_c = task.eat_immediate()?;
  let val_a = task.vm.reg_file[ra];
  let val_b = task.vm.reg_file[rb];
  let addr = (val_b + val_c) as usize;
  task.vm.write_block(addr, val_a)?;
  Ok(())
}

fn mrmovq(task: &mut Task<'_, '_, impl Region>) -> Result<(), Error> {
  let byte = task.eat()?;
  let ra = Register::try_from(byte >> 4)?; // dest
  let rb = Register::try_from(byte & 0xf)?; // base
  let val_c = task.eat_immediate()?;
  let val_b = task.vm.reg_file[rb];
  let addr = (val_b + val_c) as usize;
  let val_m = task.vm.read_block(addr)?;
  task.vm.reg_file[ra] = val_m;
  Ok(())
}

fn opq(task: &mut Task<'_, '_, impl Region>, fun: OpFun) -> Result<(), Error> {
  let byte = task.eat()?;
  let ra = Register::try_from(byte >> 4)?; // src
  let rb = Register::try_from(byte & 0xf)?; // dest
  let val_a = task.vm.reg_file[ra];
  let val_b = task.vm.reg_file[rb];

  let (result, of) = match fun {
    OpFun::Add => val_b.overflowing_add(val_a),
    OpFun::Sub => val_b.overflowing_sub(val_a),
    OpFun::And => (val_b & val_a, false),
    OpFun::Xor => (val_b ^ val_a, false),
    OpFun::Mul => val_b.overflowing_mul(val_a),
    OpFun::Div => {
      if val_a == 0 {
        return Err(Error::DivisionByZero);
      }
      (val_b / val_a, false)
    }
    OpFun::Mod => {
      if val_a == 0 {
        return Err(Error::DivisionByZero);
      }
      (val_b % val_a, false)
    }
  };

  task.vm.reg_file[rb] = result;
  task.vm.reg_file[Flag::ZF] = result == 0;
  task.vm.reg_file[Flag::SF] = result < 0;
  task.vm.reg_file[Flag::OF] = of;

  Ok(())
}

fn jxx(task: &mut Task<'_, '_, impl Region>, cond: JCmovFun) -> Result<(), Error> {
  let dest = task.eat_immediate()? as usize;
  if task.vm.reg_file.eval_condition(&cond) {
    task.vm.ip = dest;
  }
  Ok(())
}

fn call(task: &mut Task<'_, '_, impl Region>) -> Result<(), Error> {
  let dest = task.eat_immediate()? as usize;
  let val_p = task.vm.ip as Block;
  let val_rsp = task.vm.reg_file[Register::Rsp];
  let new_rsp = val_rsp - 8;
  // push ret address onto stack
  task.vm.write_block(new_rsp as usize, val_p)?;
  task.vm.reg_file[Register::Rsp] = new_rsp;
  task.vm.ip = dest;
  Ok(())
}

fn ret(task: &mut Task<'_, '_, impl Region>) -> Result<(), Error> {
  let val_rsp = task.vm.reg_file[Register::Rsp];
  let ret_addr = task.vm.read_block(val_rsp as usize)? as usize;
  let new_rsp = val_rsp + 8;
  task.vm.reg_file[Register::Rsp] = new_rsp;
  task.vm.ip = ret_addr;
  Ok(())
}

fn pushq(task: &mut Task<'_, '_, impl Region>) -> Result<(), Error> {
  let byte = task.eat()?;
  let ra = Register::try_from(byte >> 4)?; // src
  let val_a = task.vm.reg_file[ra];
  let val_rsp = task.vm.reg_file[Register::Rsp];
  let new_rsp = val_rsp - 8;
  task.vm.write_block(new_rsp as usize, val_a)?;
  task.vm.reg_file[Register::Rsp] = new_rsp;
  Ok(())
}

fn popq(task: &mut Task<'_, '_, impl Region>) -> Result<(), Error> {
  let byte = task.eat()?;
  let ra = Register::try_from(byte >> 4)?; // dest
  let val_rsp = task.vm.reg_file[Register::Rsp];
  let val_m = task.vm.read_block(val_rsp as usize)?;
  let new_rsp = val_rsp + 8;
  task.vm.reg_file[ra] = val_m;
  task.vm.reg_file[Register::Rsp] = new_rsp;
  Ok(())
}
