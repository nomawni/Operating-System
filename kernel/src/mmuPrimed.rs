const PAGE_SIZE: usize = 4096;
const PAGE_TABLE_ENTRIES: usize = 512;

#[repr(u8)]
#[derive(Clone, Copy)]
enum PageTableEntryFlags {
    VALID = 0b1,
    READABLE = 0b10,
    WRITABLE = 0b100,
    EXECUTABLE = 0b1000,
    USER = 0b10000,
}

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

struct PageTableManager {
    root_table: PageTable,
}

impl PageTableManager {
    fn new() -> Self {
        Self {
            root_table: PageTable::new(),
        }
    }
    fn map_page(&mut self, virtual_address: usize, physical_address: usize) {
        let vpn = virtual_address >> 12;
        let ppn = physical_address >> 12;

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
            llvm_asm!("sfence.vma" :: "r"(virtual_address) :: "volatile");
        }
    }

    fn unmap_page(&mut self, virtual_address: usize) {
        let vpn = virtual_address >> 12;

        let root_entry = self.root_table.get_entry_mut(vpn >> 9).unwrap();
        let table_address = root_entry.get_physical_address();

        let table = unsafe { &mut *(table_address as *mut PageTable) };

        let entry = table.get_entry_mut(vpn & 0x1ff).unwrap();
        entry.set_valid(false);

        unsafe {
            llvm_asm!("sfence.vma" :: "r"(virtual_address) :: "volatile");
        }
    }

    fn translate(&self, virtual_address: usize) -> Option<usize> {
        let vpn = virtual_address >> 12;

        let root_entry = self.root_table.get_entry(vpn >> 9)?;
        let table_address = root_entry.get_physical_address();

        let table = unsafe { &*(table_address as *const PageTable) };

        let entry = table.get_entry(vpn & 0x1ff)?;
        let physical_address = entry.get_physical_address();

        Some((physical_address << 12) | (virtual_address & 0xfff))
    }

    fn activate(&self) {
        let page_table = &self.root_table as *const _ as usize;

        unsafe {
            llvm_asm!(
            "csrw 0x180, $0" :: "r"(0) :: "volatile");
            llvm_asm!(
            "csrw 0x180, $0" :: "r"(page_table >> 12) :: "volatile");
        }
    }
}
