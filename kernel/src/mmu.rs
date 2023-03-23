#![feature(asm)]

use core::arch::asm;
const PAGE_SIZE: usize = 4096;
const PAGE_TABLE_ENTRIES: usize = 512;
#[repr(u8)]
#[derive(Clone)]
enum PageTableEntryFlags {
    VALID = 0b1,
    READABLE = 0b10,
    WRITABLE = 0b100,
    EXECUTABLE = 0b1000,
    USER = 0b10000,
}

#[derive(Clone, Copy)]
struct PageTableEntry {
    entry: usize,
}

impl PageTableEntry {
    fn is_valid(&self) -> bool {
        (self.entry & PageTableEntryFlags::VALID as usize) != 0
    }
    fn set_valid(&mut self, valid: bool) {
        if valid {
            self.entry |= PageTableEntryFlags::VALID as usize;
        } else {
            self.entry &= !(PageTableEntryFlags::VALID as usize);
        }
    }

    fn is_readable(&self) -> bool {
        (self.entry & PageTableEntryFlags::READABLE as usize) != 0
    }

    fn set_readable(&mut self, readable: bool) {
        if readable {
            self.entry |= PageTableEntryFlags::READABLE as usize;
        } else {
            self.entry &= !(PageTableEntryFlags::READABLE as usize);
        }
    }

    fn is_writable(&self) -> bool {
        (self.entry & PageTableEntryFlags::WRITABLE as usize) != 0
    }

    fn set_writable(&mut self, writable: bool) {
        if writable {
            self.entry |= PageTableEntryFlags::WRITABLE as usize;
        } else {
            self.entry &= !(PageTableEntryFlags::WRITABLE as usize);
        }
    }

    fn is_executable(&self) -> bool {
        (self.entry & PageTableEntryFlags::EXECUTABLE as usize) != 0
    }

    fn set_executable(&mut self, executable: bool) {
        if executable {
            self.entry |= PageTableEntryFlags::EXECUTABLE as usize;
        } else {
            self.entry &= !(PageTableEntryFlags::EXECUTABLE as usize);
        }
    }

    fn is_user(&self) -> bool {
        (self.entry & PageTableEntryFlags::USER as usize) != 0
    }

    fn set_user(&mut self, user: bool) {
        if user {
            self.entry |= PageTableEntryFlags::USER as usize;
        } else {
            self.entry &= !(PageTableEntryFlags::USER as usize);
        }
    }

    fn get_physical_address(&self) -> usize {
        self.entry & !(PageTableEntryFlags::VALID as usize)
    }

    fn set_physical_address(&mut self, physical_address: usize) {
        self.entry = physical_address | (self.entry & (PageTableEntryFlags::VALID as usize));
    }
}

struct PageTable {
    entries: [PageTableEntry; PAGE_TABLE_ENTRIES],
}

impl PageTable {
    fn new() -> Self {
        Self {
            entries: [PageTableEntry { entry: 0 }; PAGE_TABLE_ENTRIES],
        }
    }
    fn get_entry(&self, vpn: usize) -> Option<&PageTableEntry> {
        if vpn < PAGE_TABLE_ENTRIES {
            let entry = &self.entries[vpn];
            if entry.is_valid() {
                Some(entry)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn get_entry_mut(&mut self, vpn: usize) -> Option<&mut PageTableEntry> {
        if vpn < PAGE_TABLE_ENTRIES {
            let entry = &mut self.entries[vpn];
            if entry.is_valid() {
                Some(entry)
            } else {
                None
            }
        } else {
            None
        }
    }
}
pub struct PageTableManager {
    root_table: PageTable,
}

impl PageTableManager {
    pub fn new() -> Self {
        Self {
            root_table: PageTable::new(),
        }
    }

    fn map_page(&mut self, virtual_address: usize) {
        let ppn = allocate_page() >> 12;

        let vpn = virtual_address >> 12;
        let root_entry = self.root_table.get_entry_mut(vpn >> 9).unwrap();
        let table_address = root_entry.get_physical_address();
        let table = unsafe { &mut *(table_address as *mut PageTable) };

        let entry = table.get_entry_mut(vpn & 0x1ff).unwrap();

        entry.set_physical_address(
            (ppn << 12)
                | PageTableEntryFlags::VALID as usize
                | PageTableEntryFlags::READABLE as usize
                | PageTableEntryFlags::WRITABLE as usize,
        );

        unsafe {
            asm!("sfence.vma {}", in(reg) virtual_address);
        }
    }

    fn unmap_page(&mut self, virtual_address: usize) {
        let vpn = virtual_address >> 12;

        let root_entry = self.root_table.get_entry_mut(vpn >> 9).unwrap();

        let table_address = root_entry.get_physical_address();
        let table = unsafe { &mut *(table_address as *mut PageTable) };

        let entry = table.get_entry_mut(vpn & 0x1ff).unwrap();
        // set the entry as invalid
        entry.set_valid(false);

        unsafe {
            asm!("sfence.vma {}", in(reg) virtual_address);
        }
    }
    fn translate(&self, virtual_address: usize) -> Option<usize> {
        let vpn = virtual_address >> 12;
        let root_entry = self.root_table.get_entry(vpn >> 9)?;
        let table_address = root_entry.get_physical_address();
        let table = unsafe { &*(table_address as *const PageTable) };

        let entry = table.get_entry(vpn & 0x1ff)?;

        if !entry.is_valid() {
            return None;
        }
        let physical_address = entry.get_physical_address();

        Some((physical_address << 12) | (virtual_address & 0xfff))
    }
}

// Function to allocate a page
fn allocate_page() -> usize {
    // Code to allocate a page
    // ...
    0 // Placeholder value
}

/*
// Enable MMU in mstatus register
let mut mstatus: usize;
unsafe { llvm_asm!("csrr $0, mstatus" : "=r"(mstatus) ::: "volatile") };
mstatus |= 1 << 3; // set bit 3 to enable MMU
unsafe { llvm_asm!("csrw mstatus, $0" :: "r"(mstatus) :: "volatile") };


// Set up page table for kernel
let page_table = // pointer to page table

// Set the satp register to enable paging
let satp: usize = (8 << 60) | (page_table >> 12);
unsafe { llvm_asm!("csrw satp, $0" :: "r"(satp) :: "volatile") };

// Set sscratch to point to the supervisor stack
let stack_top = // top of the supervisor stack
unsafe { llvm_asm!("mv sscratch, $0" :: "r"(stack_top) :: "volatile") };

// Set sepc to the entry point of the kernel
let kernel_entry_point = // entry point of the kernel
unsafe { llvm_asm!("mv sepc, $0" :: "r"(kernel_entry_point) :: "volatile") };

// Enable supervisor mode
unsafe { llvm_asm!("mret") }; */
