fn factory() -> unsafe fn(i32) -> i32 {
    let make = || {
        return left::unchecked_add;
    };
    let safe: fn(i32) -> i32 = make();
    safe
}

fn main() {
    println!("unsafe-closure={}", unsafe { factory()(25) });
}
