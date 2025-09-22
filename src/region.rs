pub trait Region {
  fn instructions(&self) -> &[u8];
}

pub struct Chunk {
  instructions: Vec<u8>,
}

impl Region for Chunk {
  fn instructions(&self) -> &[u8] {
    &self.instructions
  }
}

impl From<Vec<u8>> for Chunk {
  fn from(instructions: Vec<u8>) -> Self {
    Self { instructions }
  }
}
