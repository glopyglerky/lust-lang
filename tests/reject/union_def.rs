// expect: L0602
union U {
    a: u32,
    b: f32,
}

fn main() {
    let _u = U { a: 1 };
    println!("hi");
}
