pub struct S(pub u8);

#[macro_export]
macro_rules! sum_ones {
    (@one $x:expr) => (1u8);
    ($($x:expr),*) => ($crate::S(0u8 $(+ $crate::sum_ones!(@one $x))*));
}

#[macro_export]
macro_rules! helper {
    () => (40u8);
}

#[macro_export]
macro_rules! outer {
    () => ($crate::helper!() + 2);
}
