fn main() {
    // The runner passes one program argument (fixture.txt `args=`), so the
    // count is 2 in both builds.
    println!("extra bin argc={}", std::env::args().count());
}
