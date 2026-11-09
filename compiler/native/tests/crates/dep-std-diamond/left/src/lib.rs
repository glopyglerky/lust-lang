use std::collections::VecDeque;
use std::fmt;
use std::time::Duration;

#[derive(Debug)]
pub struct TooLong(pub Duration);

impl fmt::Display for TooLong {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "too long: {:?}", self.0)
    }
}

impl std::error::Error for TooLong {}

pub fn schedule(n: u32) -> Result<VecDeque<Duration>, TooLong> {
    let mut q = VecDeque::new();
    for i in 0..n {
        q.push_back(base::step() * i);
    }
    match q.back() {
        Some(&d) if d > Duration::from_secs(1) => Err(TooLong(d)),
        _ => Ok(q),
    }
}
