fn main() {
    let a: Arr<2> = Arr { data: [1, 2] };
    println!("{}", a.data[0]);
}

struct Arr<const N: usize> {
    data: [i64; N],
}
