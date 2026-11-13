// expect: L0107
// rustc E0107: Box::new_uninit takes no generic arguments (write Box::<T>::new_uninit()).
fn main() { let b = Box::new_uninit::<u8>(); drop(b); }
