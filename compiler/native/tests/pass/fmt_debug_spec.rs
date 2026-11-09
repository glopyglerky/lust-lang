// Debug through the Formatter: width, fill, sign, and zero flags reach
// every leaf of a compound value, as Rust's derived and std impls pass
// the Formatter down; str, char, and unit-like names ignore width.
#[derive(Debug, Clone, Copy)]
enum Shape {
    Dot,
    Line(i32),
    Rect { w: u8, h: u8 },
}

#[derive(Debug)]
struct Unit;

#[derive(Debug)]
struct Pair(i32, f64);

#[derive(Debug)]
struct Rec {
    id: u32,
    name: &'static str,
    tags: Vec<u8>,
    shape: Option<Shape>,
}

fn main() {
    println!("[{:5?}] [{:<5?}] [{:^7?}] [{:>6?}]", Some(1), Some(2), Some(3), Ok::<i32, ()>(4));
    println!("[{:8?}] [{:<8?}] [{:08?}] [{:+?}]", Shape::Dot, Shape::Line(5), Shape::Line(-5), Shape::Line(6));
    println!("[{:4?}]", Shape::Rect { w: 1, h: 22 });
    println!("[{:6?}] [{:<6?}] [{:06?}]", Unit, Pair(7, 1.5), Pair(-8, -0.25));
    println!("[{:3?}] [{:>4?}] [{:+05?}]", vec![1, 2, 3], [10u8, 20], (1, -2));
    println!("[{:9?}] [{:<9?}] [{:^9?}] [{:9?}]", "s", 'c', "a\nb", '\t');
    println!("[{:8?}] [{:<8?}] [{:^8?}] [{:08?}]", true, false, (), 1.5);
    println!("[{:.2?}] [{:8.3?}] [{:<10?}] [{:+?}]", 1.0, 2.25f32, f64::NAN, -0.0);
    println!("[{:?}] [{:?}] [{:?}] [{:?}]", 1e21f64, 1e-7f64, 1e16f64, 123456789.0f32);
    println!("[{:5?}]", Rec { id: 9, name: "n\"q", tags: vec![1, 2], shape: Some(Shape::Rect { w: 3, h: 4 }) });
    let b = Some(Box::new(5));
    let rb = &b;
    let r = &rb;
    println!("[{:4?}] [{:?}]", r, Box::new(Pair(1, 2.0)));
    println!("[{:-}] [{:-?}] [{:-5}]", 5, -5, 7);
}
