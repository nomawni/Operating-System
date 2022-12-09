#![allow(dead_code)]
use core::arch::asm;
use riscv_utils as riscv;
use riscv_utils::SysCall;

unsafe fn system_call(syscall: SysCall, param_0: u64, param_1: u64) -> u64 {
    let number = syscall as u64;
    riscv::write_function_reg!(
        number => "a7",
        param_0 => "a0",
        param_1 => "a1"
    );
    asm!("ecall");
    let output;
    riscv::read_function_reg!("a0" => output);
    output
}

pub fn exit() {
    unsafe {
        system_call(SysCall::Exit, 0, 0);
    }
}
pub fn sys_yield() {
    unsafe {
        system_call(SysCall::Yield, 0, 0);
    }
}
