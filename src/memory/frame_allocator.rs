use spin::Mutex;
use super::MAX_PHYSICAL_PAGES;

pub const MAX_BITS: usize = MAX_PHYSICAL_PAGES/8 + 1;


pub struct SegFrameAlloc {
    vis: [u8; MAX_BITS],
}

impl SegFrameAlloc {
    pub fn init(&mut self, l: usize, r: usize) {
        //println!("{}, {}, {}, {}", MAX_BITS, MAX_PHYSICAL_PAGES, l, r);
        for i in l..r {
            let idx = i / 8;
            let offset = i % 8;
            self.vis[idx] |= (1 << offset) as u8;
        }
    }

    pub fn alloc(&mut self) -> usize {
        for i in 0..MAX_PHYSICAL_PAGES {
            let idx = i / 8;
            let offset = i % 8;
            if (self.vis[idx] & (1 << offset) as u8) != 0 {
                self.vis[idx] ^= ((1 << offset) as u8);
                return i;
            }
        }
        panic!("no page to alloc");
    }

    pub fn dealloc(&mut self, n: usize) {
        let idx = n / 8;
        let offset = n % 8;
        self.vis[idx] |= (1 << offset) as u8;
    }
}

pub static SEG_FRAME_ALLOC: Mutex<SegFrameAlloc>= Mutex::new(SegFrameAlloc { vis: [0; MAX_BITS], });