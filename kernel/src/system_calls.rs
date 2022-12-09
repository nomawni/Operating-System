pub use core::arch::asm;
use riscv_utils::*;

use crate::scheduler;

fn syscall_from(number: usize) -> SysCall {
    crate::enum_matching!(number: SysCall::Yield, SysCall::Exit);
    panic!("Illegal syscall: {}", number);
}

pub unsafe fn syscall(number: usize, _param_0: usize, _param_1: usize) -> Option<usize> {
    match syscall_from(number) {
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

unsafe fn sys_yield() {
    let next =
        scheduler::next().expect("No next user prog for system yield. Idle task not implemented");
    scheduler::switch(next);
}
