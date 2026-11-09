// alloc's boxed-array TryFrom pairs: `Box<[T]>` and `Vec<T>` into
// `Box<[T; N]>`, the source back in `Err` on a length miss (ts-rust
// core.rs full_chunk: `values.into_boxed_slice().try_into()`), with Drop
// elements dropped once, ZSTs, an empty array, and the path forms.
#[derive(Debug)]
struct D(u8);

impl Drop for D {
    fn drop(&mut self) {
        println!("drop {}", self.0);
    }
}

type Chunk<T> = [T; 3];

fn full<T>(values: Vec<T>) -> Box<Chunk<T>> {
    values.into_boxed_slice().try_into().unwrap_or_else(|_| panic!("a chunk holds 3 values"))
}

fn main() {
    let b = full(vec![D(1), D(2), D(3)]);
    println!("{:?}", b);
    let mut v = Vec::with_capacity(8);
    v.extend([D(4), D(5)]);
    let r: Result<Box<[D; 3]>, Vec<D>> = v.try_into();
    match r {
        Ok(_) => println!("ok"),
        Err(back) => println!("err {}", back.len()),
    }
    let s: Box<[String]> = vec![String::from("a")].into();
    let one: Result<Box<[String; 1]>, Box<[String]>> = s.try_into();
    println!("{:?}", one);
    let units: Box<[(); 2]> = vec![(), ()].try_into().unwrap();
    let empty: Box<[u8; 0]> = Vec::new().try_into().unwrap();
    println!("{:?} {:?}", units, empty);
    println!("{:?} {:?}", <Box<[u8; 2]>>::try_from(vec![7u8, 8]), Box::<[u8; 1]>::try_from(vec![9u8, 9]));
    drop(b);
    println!("end");
}
