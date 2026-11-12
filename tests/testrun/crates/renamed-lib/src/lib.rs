#![no_std]
#![forbid(unsafe_code)]

pub const fn increment(value: u32) -> u32 {
    value + 1
}

#[cfg(test)]
mod tests;
