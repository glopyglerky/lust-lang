// capability: rejected: `async` functions and blocks
// expect: L0405

async fn f() -> u32 {
    1
}

fn main() {
    let _ = f();
}
