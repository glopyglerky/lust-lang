mod doctor;

fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    let args = if args.first().is_some_and(|arg| arg == "lust") { &args[1..] } else { &args[..] };
    if args == [std::ffi::OsString::from("doctor")] {
        std::process::exit(doctor::run());
    }
    std::process::exit(cargo_lust::run(args));
}
