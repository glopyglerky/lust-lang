#[cfg(any())]
fn f() {
    println!(1 +);
    undefined_macro! { totally not expressions }
    let v = vec![1, +];
}

#[cfg(any())]
undefined_macro! { 1 + }

fn main() {
    println!("ok");
}
