// feature: async.await
// form: async fn and .await in an async block, as in ts-rust project/ata
async fn fetch() -> i32 { let x = async { 2 }.await; x + 1 }
fn main() { let _fut = fetch(); println!("ok"); }
