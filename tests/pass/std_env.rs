// std::env (W-SHIM): var and var_os on set and unset names, VarError
// texts, vars and vars_os, args_os, temp_dir, and current_dir.
use std::env;
use std::ffi::OsString;
fn main() {
    println!("{:?}", env::var("PATH"));
    println!("{:?}", env::var("LUST_PROBE_MISSING"));
    println!("{:?}", env::var_os("PATH"));
    println!("{:?}", env::var_os(""));
    println!("{:?}", env::var("LUST_SHIM_TEST_UNSET_E"));
    match env::var("NOPE_X") { Ok(v) => println!("ok {}", v), Err(e) => println!("err {} / {:?}", e, e) }
    let mut n = 0;
    for (k, v) in env::vars() { if k == "PATH" { println!("{}={}", k, v); n += 1; } }
    for (k, v) in env::vars_os() { if k == *"PATH" { println!("os {:?} {:?}", k, v); } }
    println!("{}", n);
    let mut c = 0;
    for a in env::args_os() { if c > 0 { println!("arg {:?}", a); } c += 1; }
    println!("{}", env::temp_dir().is_absolute());
    match env::current_dir() { Ok(d) => println!("{}", d.is_absolute()), Err(e) => println!("{}", e) }
    let o: OsString = "x".into();
    println!("{:?} {}", o, match env::current_exe() { Ok(_) => true, Err(_) => false });
}
