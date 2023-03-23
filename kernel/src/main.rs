#![no_std]
#![no_main]

mod asm;
mod exception_handler;
mod hardware;
mod macros;
mod mmu;
mod page;
mod panic_handler;
mod scheduler;
mod setup;
mod system_calls;
mod uart;
mod user_prog;

pub(crate) use macros::*;

//TODO implement shutdown
fn _shutdown() {}

#[no_mangle]
unsafe extern "C" fn kernel_setup() {
    // Initalize the heap
    page::PageAllocator::init_heap();
    setup::setup();

    // switch to user mode (configured in mstatus) and jump to address in mepc CSR -> main().

    // Initialize the UART writer with the appropriate base address
    uart::print_os();

    // Test the print and println functions
    //println!("Hello, world!");
    //print!("Cycle count: {}\n", mcycle::read());

    // Allocate a page
    let mut allocator = page::PageAllocator::new();
    // The MMU
    let mut manager = mmu::PageTableManager::new();
    let virtual_address = 0xdeadbeefusize;
    let physical_address = allocator.allocate_kernel_page();
    // The rest of the program
    let user1 = scheduler::init_prog(user_prog::USER1);
    scheduler::init_prog(user_prog::USER2);
    scheduler::boot_prog(user1);
}
