const PAGE_SIZE: usize = 4096;
const MEM_SIZE: usize = 128 * 1024 * 1024;

const KERNEL_PAGES: usize = 16384;
const BITMAP_SIZE: usize = (MEM_SIZE / PAGE_SIZE + 7) / 8;

static mut ALLOC_START: usize = 0;
static mut ALLOC_END: usize = 0;

pub struct PageAllocator {
    kernel_bitmap: [u8; BITMAP_SIZE],
    user_bitmap: [u8; BITMAP_SIZE],
    kernel_end: usize,
    user_end: usize,
    allocated_kernel_pages: [usize; KERNEL_PAGES],
    allocated_user_pages: [usize; MEM_SIZE / PAGE_SIZE - KERNEL_PAGES],
    kernel_pages_count: usize,
    user_pages_count: usize,
}

impl PageAllocator {
    pub fn new() -> Self {
        Self {
            kernel_bitmap: [0; BITMAP_SIZE],
            user_bitmap: [0; BITMAP_SIZE],

            kernel_end: KERNEL_PAGES * PAGE_SIZE,
            user_end: 0,

            allocated_kernel_pages: [0; KERNEL_PAGES],
            allocated_user_pages: [0; MEM_SIZE / PAGE_SIZE - KERNEL_PAGES],

            kernel_pages_count: 0,
            user_pages_count: 0,
        }
    }

    fn allocate_page(&mut self, kernel: bool) -> Option<usize> {
        let start = if kernel { 0 } else { self.kernel_end };
        let end = if kernel { self.kernel_end } else { MEM_SIZE };

        let bitmap = if kernel {
            &mut self.kernel_bitmap
        } else {
            &mut self.user_bitmap
        };

        for i in (start / PAGE_SIZE / 8)..(end / PAGE_SIZE / 8) {
            if bitmap[i] != 0xff {
                let free_bit_index = bitmap[i].trailing_zeros() as usize;

                let page_index = i * 8 + free_bit_index;

                bitmap[i] |= 1 << free_bit_index;

                if !kernel && page_index * PAGE_SIZE > self.user_end {
                    self.user_end = page_index * PAGE_SIZE;
                }

                if kernel {
                    self.allocated_kernel_pages[self.kernel_pages_count] = page_index * PAGE_SIZE;
                    self.kernel_pages_count += 1;
                } else {
                    self.allocated_user_pages[self.user_pages_count] = page_index * PAGE_SIZE;
                    self.user_pages_count += 1;
                }

                return Some(page_index * PAGE_SIZE);
            }
        }
        // No free page was found
        None
    }

    fn free_page(&mut self, phys_addr: usize, kernel: bool) {
        let bitmap = if kernel {
            &mut self.kernel_bitmap
        } else {
            &mut self.user_bitmap
        };
        // Mark the page as free in the bitmap
        let page_index = phys_addr / PAGE_SIZE;
        let bitmap_index = page_index / 8;
        let bit_index = page_index % 8;
        bitmap[bitmap_index] &= !(1 << bit_index);
        // Update the end address if necessary
        if kernel && phys_addr >= self.kernel_end && phys_addr < self.user_end {
            self.user_end = self.kernel_end;
        } else if !kernel && phys_addr >= self.user_end {
            self.user_end = phys_addr;
        }
        // Remove the physical address from the list of allocated pages
        if kernel {
            let mut i = 0;
            while i < self.kernel_pages_count {
                if self.allocated_kernel_pages[i] == phys_addr {
                    break;
                }
                i += 1;
            }
            while i < self.kernel_pages_count - 1 {
                self.allocated_kernel_pages[i] = self.allocated_kernel_pages[i + 1];
                i += 1;
            }
            self.allocated_kernel_pages[self.kernel_pages_count - 1] = 0;
            self.kernel_pages_count -= 1;
        } else {
            let mut i = 0;
            while i < self.user_pages_count {
                if self.allocated_user_pages[i] == phys_addr {
                    break;
                }
                i += 1;
            }
            while i < self.user_pages_count - 1 {
                self.allocated_user_pages[i] = self.allocated_user_pages[i + 1];
                i += 1;
            }
            self.allocated_user_pages[self.user_pages_count - 1] = 0;
            self.user_pages_count -= 1;
        }
    }
    pub fn allocate_kernel_page(&mut self) -> Option<usize> {
        self.allocate_page(true)
    }

    pub fn allocate_user_page(&mut self) -> Option<usize> {
        self.allocate_page(false)
    }

    pub fn free_kernel_page(&mut self, phys_addr: usize) {
        self.free_page(phys_addr, true)
    }

    pub fn free_user_page(&mut self, phys_addr: usize) {
        self.free_page(phys_addr, false)
    }

    #[no_mangle]
    pub extern "C" fn init_heap() {
        let kernel_end = KERNEL_PAGES * PAGE_SIZE;
        let heap_start = ((kernel_end - 1) / PAGE_SIZE + 1) * PAGE_SIZE;
        let heap_end = (MEM_SIZE / PAGE_SIZE) * PAGE_SIZE - PAGE_SIZE;

        unsafe {
            ALLOC_START = heap_start;
            ALLOC_END = heap_end;
        }
    }
}
