fn main() {
    println!("{}", concat!("a", 1, true, 'c', -3, "z"));
    println!("{}", concat!(1u8, 2i64, 0x10, 1_000));
    println!("{}", concat!(1.5, " ", 2.0, " ", 1e3, " ", 1.0f32));
    println!("{} {} {}", cfg!(unix), cfg!(windows), cfg!(target_pointer_width = "64"));
    println!("{} {}", cfg!(panic = "abort"), cfg!(panic = "unwind"));
    println!("{} {}", cfg!(target_endian = "little"), cfg!(target_has_atomic = "64"));
}
