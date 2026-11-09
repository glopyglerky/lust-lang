macro_rules! twice {
    ($e:expr) => {
        $e * 2
    };
}

fn main() {
    println!("{}", twice!(2));
}
