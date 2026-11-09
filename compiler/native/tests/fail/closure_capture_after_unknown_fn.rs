// expect: L0101
// rustc E0425: `with_pool` does not exist. The closure's captures then have unknown types;
// lust reports only the cause, not a follow-on L0820 (ts-rust deep census, program.rs).
fn main() {
    with_pool(|pool| {
        let token = pool.token.clone();
        let job = move || drop(token);
        job();
    });
}
