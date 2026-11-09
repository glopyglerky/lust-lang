// Outside reads in a module file whose own text never changes: only the
// included files and the client env do.
pub fn text() -> &'static str {
    include_str!("data.txt")
}
pub fn env() -> Option<&'static str> {
    option_env!("LUST_SCEN_INCL")
}
pub fn n() -> u32 {
    include!("n.in")
}
