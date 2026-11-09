// A multiply-assigned local is only known inside its own block, so
// none of these fires rustc's arithmetic_overflow lint. The fns are
// not called: calling them would panic at run time.
fn f() {}

#[allow(dead_code)]
fn after_call() {
    let mut a: u8 = 255;
    f();
    a += 1;
}

#[allow(dead_code)]
fn in_branch() {
    let mut a: u8 = 255;
    if true {
        a += 1;
    }
}

#[allow(dead_code)]
fn plain_after_call() -> u8 {
    let mut a: u8 = 0;
    a = 255;
    f();
    a + 1
}

#[allow(dead_code)]
fn in_index_place() {
    let mut a = [255u8];
    a[0] += 1;
}

fn main() {
    println!("ok");
}
