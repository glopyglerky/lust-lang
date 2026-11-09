// A collect error keeps the per-slot caches: the next request walks every
// passing body (Front::walk_after_reject). v2 changes f's return type and
// adds a bad where clause (a collect error: no body checks run). v3 drops
// the clause only: g was last checked against v1's f and must fail now.
// v4 goes back to v1; v5 rejects again; v6 changes other's signature.
// Every version keeps the same lines above the edit points.
fn f() -> u32 {
    1
}
fn g() -> u32 {
    f() + 1
}
fn other(_x: u32) -> u32 {
    7
}
fn main() {
    println!("{} {}", g(), other(2));
}

