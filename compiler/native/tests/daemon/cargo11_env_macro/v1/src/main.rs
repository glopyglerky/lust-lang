mod m;
fn main() {
    println!("{}", env!("LUST_SCEN_E"));
    println!("{:?} {:?}", option_env!("LUST_SCEN_O"), m::o());
}
