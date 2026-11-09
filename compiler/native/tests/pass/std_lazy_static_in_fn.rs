// A LazyLock static inside a fn body: the hidden init fn the lazy
// rewrite makes is a body item, and the const evaluator now finds
// it through the static's block scope. Two fns get distinct statics.
use std::sync::atomic::{AtomicU32, Ordering};

static CNT: AtomicU32 = AtomicU32::new(0);

fn a() -> u32 {
    static L: std::sync::LazyLock<u32> = std::sync::LazyLock::new(|| CNT.fetch_add(1, Ordering::SeqCst) + 10);
    *L
}

fn b() -> u32 {
    static L: std::sync::LazyLock<u32> = std::sync::LazyLock::new(|| CNT.fetch_add(1, Ordering::SeqCst) + 100);
    *L
}

fn main() {
    println!("{} {} {} {} {}", a(), b(), a(), b(), CNT.load(Ordering::SeqCst));
}
