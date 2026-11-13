// expect: L0110
// A user type named ExitCode is not a Termination type.
struct ExitCode(u8);
fn main() -> ExitCode {
    ExitCode(3)
}
