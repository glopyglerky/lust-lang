// feature: macros.std_missing
// form: concat!, stringify!, env!, include_str!, line! and file! by length
fn main() {
    let joined = concat!("a", "b");
    let src = include_str!("imp_helper.rs");
    println!(
        "{} {} {} {} {}",
        joined.len(),
        stringify!(some_name).len(),
        !env!("PATH").is_empty(),
        src.len(),
        line!() > 0 && !file!().is_empty()
    );
}
