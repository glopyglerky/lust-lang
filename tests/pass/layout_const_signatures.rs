#![no_std]
extern crate std;

use core::mem::{align_of as alignment, size_of as extent};
use core::mem as memory;

type Word = u32;
type TupleBytes = [u8; core::mem::size_of::<(u16, u32)>()];

struct Buffer {
    bytes: [u8; alignment::<[u64; 2]>()],
}

fn word() -> [u8; extent::<Word>()] { [0; 4] }
fn array() -> [u8; memory::size_of::<[u16; 3]>()] { [0; 6] }
fn fixed<T>() -> [u8; core::mem::size_of::<u32>()] { [0; 4] }

const fn size_of<T>() -> usize { 3 }
fn own_function() -> [u8; size_of::<u64>()] { [0; 3] }

mod shadow {
    pub mod std {
        pub mod mem {
            pub const fn size_of<T>() -> usize { 5 }
        }
    }
    pub mod core {
        pub mod mem {
            pub const fn align_of<T>() -> usize { 7 }
        }
    }
    pub fn size() -> [u8; std::mem::size_of::<u64>()] { [0; 5] }
    pub fn align() -> [u8; core::mem::align_of::<u64>()] { [0; 7] }
}

fn main() {
    let tuple: TupleBytes = [0; 8];
    let buffer = Buffer { bytes: [0; 8] };
    assert_eq!((word().len(), array().len(), fixed::<u8>().len(), tuple.len(),
        buffer.bytes.len(), own_function().len(), shadow::size().len(), shadow::align().len()),
        (4, 6, 4, 8, 8, 3, 5, 7));
}
