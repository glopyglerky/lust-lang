fn main() {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    std::process::exit(cargo_lust::run_cli(&args));
}
