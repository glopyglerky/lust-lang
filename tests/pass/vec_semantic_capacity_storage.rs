use std::sync::{Mutex, OnceLock};

struct Nested {
    value: (Mutex<()>, [u8; 1016]),
}

fn main() {
    let mut locks = Vec::new();
    for n in 0..5 {
        locks.push(Mutex::new([n as u8; 1016]));
        println!("lock {} {}", locks.len(), locks.capacity());
    }
    for n in 0..5 {
        let value = locks[n].lock().unwrap();
        assert_eq!(value[0], n as u8);
        assert_eq!(value[1015], n as u8);
    }

    let mut reserved: Vec<Mutex<[u8; 1016]>> = Vec::new();
    reserved.reserve(1);
    println!("lock-reserve {}", reserved.capacity());
    let mut nested: Vec<[Mutex<()>; 65]> = Vec::new();
    nested.reserve(1);
    println!("lock-array {}", nested.capacity());
    let mut named: Vec<Nested> = Vec::new();
    named.reserve(1);
    println!("named-tuple {}", named.capacity());

    // A known pointer shape inside an aggregate stays a thin pointer;
    // the bound is checked from the modeled value layout, not its pointee.
    let mut pointers: Vec<[&u8; 128]> = Vec::new();
    pointers.reserve(1);
    println!("pointer-array {}", pointers.capacity());

    let source = [[23u8; 1025]];
    let mut copied: Vec<[u8; 1025]> = Vec::new();
    source.as_slice().clone_into(&mut copied);
    println!("clone-into {} {}", copied.len(), copied.capacity());
    copied.insert(0, copied[0]);
    assert_eq!(copied[0][1024], 23);
    assert_eq!(copied[1][1024], 23);

    let mut cells = Vec::new();
    for n in 0..5 {
        let cell = OnceLock::new();
        assert!(cell.set([n as u8; 1020]).is_ok());
        cells.push(cell);
    }
    for n in 0..5 {
        assert_eq!(cells[n].get().unwrap()[1019], n as u8);
    }
    println!("cell-storage {} {}", cells.len(), cells.capacity());

    let mut zst: Vec<()> = Vec::new();
    zst.reserve(1);
    zst.push(());
    zst.insert(0, ());
    assert_eq!(zst.capacity(), usize::MAX);
    println!("zst {} {}", zst.len(), zst.capacity());
}
