extern "C" {
    static TEXT_START: usize;
    static TEXT_END: usize;
    static DATA_START: usize;
    static DATA_END: usize;
    static RODATA_START: usize;
    static RODATA_END: usize;
    static BSS_START: usize;
    static BSS_END: usize;
    static KERNEL_STACK_START: usize;
    static KERNEL_STACK_END: usize;
    static HEAP_START: usize;
    static HEAP_END: usize;
}

pub fn id_map_range(root: &mut page::Table, start: usize, end: usize, bits: i64) {
    let mut memaddr = start & !(page::PAGE_SIZE - 1);
    let num_kb_pages = (page::aling_value(end, 12) - memaddr) / page::PAGE_SIZE;

    for _ in 0..num_kb_pages {
        page::map(root, memaddr, memaddr, bits, 0);
        memaddr += 1 << 12;
    }
}

#[no_mangle]
extern "C" fn kinit() {
    uart::Uart::new(0x1000_000).init();
    page::init();
    kmem::init();

    let root_ptr = kmem::get_page_table();
    let root_u = root_ptr as usize;
    let mut root = unsafe { root_ptr.as_mut().unwrap() };
    let kheap_head = kmem::get_head() as usize;
    let total_pages = kmem::get_num_allocations();
    println!();
    println!();
    unsafe {
        println!("TEXT: 0x{:x} -> 0x{:x}", TEXT_START, TEXT_END);
        println!("RODATA : 0x{:x} -> 0x{:x}", RODATA_START, RODATA_END);
        println!("TEXT: 0x{:x} -> 0x{:x}", DATA_START, DATA_END);
        println!("TEXT: 0x{:x} -> 0x{:x}", BSS_START, BSS_END);

        println!(
            "STACK: 0x{:x} -> 0x{:x}",
            KERNEL_STACK_START, KERNEL_STACK_END
        );

        println!(
            "HEAP: 0x{:x} -> 0x{:x}",
            kheap_head,
            kheap_head + total_pages * page::PAGE_SIZE
        );
    }
    id_map_range(
        &mut root,
        kheap_head,
        kheap_head + total_pages * page::PAGE_SIZE,
        page::EntryBits::ReadWrite.val(),
    );

    unsafe {
        let num_pages = HEAP_SIZE / page::PAGE_SIZE;
        // Map heap descriptors
        id_map_range(
            &mut root,
            HEAP_START,
            HEAP_START + num_pages,
            page::EntryBits::ReadWrite.val(),
        );

        // Map executable section
        id_map_range(
            &mut root,
            TEXT_START,
            TEXT_END,
            page::EntryBits::ReadExecute.val(),
        );

        // Map rodata section

        id_map_range(
            &mut root,
            RODATA_START,
            RODATA_END,
            page::EntryBits::ReadExecute.val(),
        );

        // Map data section
        id_map_range(
            &mut root,
            DATA_START,
            DATA_END,
            page::EntryBits::ReadWrite.val(),
        );

        // Map bss section
        id_map_range(
            &mut root,
            BSS_START,
            BSS_END,
            page::EntryBits::ReadWrite.val(),
        );

        // Map kernel stack
        id_map_range(
            &mut root,
            KERNEL_STACK_START,
            KERNEL_STACK_END,
            page::EntryBits::ReadWrite.val(),
        );

        // UART
        id_map_range(
            &mut root,
            0x1000_0000,
            0x1000_0100,
            page::EntryBits::ReadWrite.val(),
        );

        // CLINT -> MSIP
        id_map_range(
            &mut root,
            0x0200_0000,
            0x0200_ffff,
            page::EntryBits::ReadWrite.val(),
        );

        // PLIC
        id_map_range(
            &mut root,
            0x0c00_0000,
            0x0c00_2000,
            page::EntryBits::ReadWrite.val(),
        );

        id_map_range(
            &mut root,
            0x0c20_0000,
            0x0c20_8000,
            page::EntryBits::ReadWrite.val(),
        );
        page::print_page_allocations();

        let satp_value = cpu::build_satp(8, 0, root_u);

        unsafe {
            cpu::mscratch_write(
                (&mut cpu::KERNEL_TRAP_FRAME[0] as *mut cpu::KernelTrapFrame) as usize,
            );

            cpu::sscratch_write(cpu::mscratch_read());
            cpu::KERNEL_TRAP_FRAME[0].satp = satp_value;

            cpu::KERNEL_TRAP_FRAME[0].trap_stack = page::zalloc(1).add(4096);

            id_map_range(
                &mut root,
                cpu::KERNEL_TRAP_FRAME[0].trap_stack as usize - 4096,
                cpu::KERNEL_TRAP_FRAME[0].trap_stack as usize,
                page::EntryBits::ReadWrite.val(),
            );

            id_map_range(
                &mut root,
                cpu::mscratch_read(),
                cpu::mscratch_read() | 0xfff,
                page::EntryBits::ReadWrite.val(),
            );

            let p = cpu::KERNEL_TRAP_FRAME[0].trap_stack as usize;
            let m = page::virt_to_phys(&root, p).unwrap_or(0);
            println!("Walk 0x{:x} = 0x{:x}", p, m);
        }

        println!("Setting 0x{:x}", satp_value);
        println!("Scratch reg = 0x{:x}", cpu::mscratch_read());
        cpu::satp_write(satp_value);
        cpu::satp_fence();
    }
}

pub mod mem;
