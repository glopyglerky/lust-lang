// Sol review probe `positive_drop_vec_refs`: rustc accepts.
fn main() {
    let values;
    { let x = 1; values = vec![&x]; }
}
