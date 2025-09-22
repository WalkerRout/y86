use y86::region::Chunk;
use y86::vm::Vm;

#[allow(dead_code)]
fn simple_add_program() -> Vec<u8> {
  #[rustfmt::skip]
  let program = vec![
    // irmovq $7, %rdi (first argument)
    0x30, 0xF7, 0x07, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    // irmovq $5, %rsi (second argument)
    0x30, 0xF6, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    // call add_two (at address 0x20)
    0x80, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    // halt
    0x00,
    
    // binary_add function (starts at address 0x20):
    // pushq %rbp
    0xA0, 0x5F,
    // rrmovq %rsp, %rbp
    0x20, 0x45,
    // rrmovq %rdi, %rax
    0x20, 0x70,
    // addq %rsi, %rax
    0x60, 0x60,
    // popq %rbp
    0xB0, 0x5F,
    // ret
    0x90,
  ];
  program
}

fn main() {
  let mut vm = Vm::new();
  let region = Chunk::from(simple_add_program());

  while let Ok(()) = vm.step(&region) {}
  dbg!(vm);
}
