pub trait T {}
impl T for u32 {}
impl dyn T + '_ {
    pub(crate) fn secret(&self) -> u32 {
        7
    }
    pub fn open(&self) -> u32 {
        8
    }
}
