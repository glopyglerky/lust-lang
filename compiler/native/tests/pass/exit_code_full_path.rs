// ExitCode by full std path: in a Vec, compared, printed, returned.
fn main() -> std::process::ExitCode {
    let v = vec![std::process::ExitCode::SUCCESS, std::process::ExitCode::from(9)];
    println!("{:?} {}", v, v[0] != v[1]);
    v[1]
}
