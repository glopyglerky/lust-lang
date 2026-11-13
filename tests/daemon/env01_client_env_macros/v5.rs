fn o() -> Option<&'static str> {
    option_env!("LUST_SCEN_O")
}
fn main() {
    println!("{}", env!("LUST_SCEN_E"));
    println!("{:?} {}", o(), 2);
}
