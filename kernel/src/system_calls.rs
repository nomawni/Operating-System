pub use core::arch::asm;
use core::ops::Add;
use riscv_utils::*;

use crate::{
    hardware::{
        memory_mapping::MemoryMapping,
        uart::{self},
    },
    scheduler,
};

fn syscall_from(number: usize) -> SysCall {
    crate::enum_matching!(
        number: SysCall::GetChar,
        SysCall::Print,
        SysCall::Yield,
        SysCall::Exit
    );
    panic!("Illegal syscall: {}", number);
}

pub unsafe fn syscall(number: usize, _param_0: usize, _param_1: usize) -> Option<usize> {
    match syscall_from(number) {
        SysCall::GetChar => {
            return sys_get_char();
        }
        SysCall::Print => {
            sys_print_string(_param_0, _param_1);
            return None;
        }
        SysCall::Exit => {
            exit();
            return None;
        }
        SysCall::Yield => {
            scheduler::cur().increment_mepc();
            sys_yield();
            return None;
        }
    }
}

unsafe fn exit() {
    let cur = scheduler::cur();
    let prog_info = cur.prog_info();
    scheduler::end_prog(scheduler::cur());
    scheduler::init_prog(prog_info);
    sys_yield();
}

unsafe fn sys_get_char() -> Option<usize> {
    return Some(uart::read_char() as usize);
}

unsafe fn sys_print_string(str_ptr: usize, size: usize) {
    // cast to u8 to increment Option<usize> to char pointer
    let mut str_ptr = str_ptr.clone();
    for _ in 0..size {
        // Read value from the pointer with MemoryMapping
        let char = MemoryMapping::<char>::new(str_ptr as usize).read();
        uart::print_char(char);
        str_ptr = str_ptr.add(1);
    }
}

unsafe fn sys_yield() {
    let next =
        scheduler::next().expect("No next user prog for system yield. Idle task not implemented");
    scheduler::switch(next);
}
