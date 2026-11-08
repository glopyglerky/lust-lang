use fixture_core::{LendingRead, Reader};
use fixture_derive::Answer;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Answer)]
struct Derived;

struct DropProbe(Arc<AtomicU32>);
impl Drop for DropProbe {
    fn drop(&mut self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

fn main() {
    std::panic::set_hook(Box::new(|_| {}));
    let drops = Arc::new(AtomicU32::new(0));
    let captured = drops.clone();
    let caught = std::panic::catch_unwind(move || {
        let _guard = DropProbe(captured);
        panic!("qualification panic");
    });
    let value = 19;
    let pointer = unsafe { fixture_core::read_raw(&value) };
    let ffi: extern "C" fn(u32) -> u32 = fixture_core::ffi_increment;
    println!(
        "derived={} generated={} async={} raw={} gat={} const={} ffi={} body={} probe={} panic={} drops={}",
        Derived::derived(),
        fixture_core::build_value(),
        fixture_core::complete(fixture_core::deferred_add(20, 22)),
        pointer,
        Reader(13).read(),
        fixture_core::array_sum([3, 5, 7]),
        ffi(20),
        fixture_core::body_probe(20),
        fixture_core::benchmark_value() as u64,
        u32::from(caught.is_err()),
        drops.load(Ordering::SeqCst),
    );
    for argument in std::env::args().skip(1) {
        println!("argument={argument}");
    }
}
