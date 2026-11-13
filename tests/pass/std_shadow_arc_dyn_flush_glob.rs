// Any std glob anywhere makes lust treat io::Write as in scope, and the
// table lists Write for Arc<T> with no check of Arc<File>-only impl.
#[allow(unused_imports)] use std::collections::*;
use std::io::Write;
use std::sync::Arc;
trait Rwc: Send + Sync {
    fn flush(&self) -> std::io::Result<()>;
}
struct S;
impl Rwc for S {
    fn flush(&self) -> std::io::Result<()> { Ok(()) }
}
fn main() {
    let a: Arc<dyn Rwc> = Arc::new(S);
    a.flush().unwrap();
    let _ = std::io::stdout().flush();
}
