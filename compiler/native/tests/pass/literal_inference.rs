fn take32(x: f32) -> f32 {
    x
}

fn main() {
    let a = 3000000000 as u64;
    let b = 4294967295 as u32;
    println!("{} {}", a, b);
    let c = 1.000000059604644775390626 as f32;
    let d: f32 = 1.00000017881393432617187499;
    println!("{} {}", c, d);
    let x = 0.1;
    let y = take32(x);
    println!("{} {}", y, x + 0.2);
    let mut v = Vec::new();
    v.push(0.1);
    let z: f32 = v[0];
    println!("{} {}", z, v[0] + 0.2);
    let big = 16777217.0;
    println!("{}", take32(big));
}
