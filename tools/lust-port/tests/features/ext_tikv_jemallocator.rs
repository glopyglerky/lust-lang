// feature: ext.tikv_jemallocator
// form: the #[global_allocator] static of ts-rust's bin roots, plus the type's derives
// cargo-dep: tikv-jemallocator = "=0.7.0"
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;
fn main() {
    let v: Vec<String> = (0..20).map(|i| format!("x{i}")).collect();
    println!("{} {:?} {:?}", v.len(), GLOBAL, tikv_jemallocator::Jemalloc::default());
}
