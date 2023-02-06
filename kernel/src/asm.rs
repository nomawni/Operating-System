use core::arch::global_asm;

global_asm!(include_str!("asm/boot.S"));
global_asm!(include_str!("asm/exception.S"));
global_asm!(include_str!("asm/mem.S"));
extern "C" {
    pub fn exception();
}
