// Structs, impl blocks with self/&self/&mut self, associated functions, nested structs.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Point {
    x: i64,
    y: i64,
}

#[derive(Debug)]
struct Path {
    name: String,
    points: Vec<Point>,
}

impl Point {
    fn new(x: i64, y: i64) -> Point {
        Point { x, y }
    }
    fn manhattan(&self, other: &Point) -> i64 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
    fn shift(&mut self, d: i64) {
        self.x += d;
        self.y -= d;
    }
}

impl Path {
    fn new(name: &str) -> Self {
        Path { name: name.to_string(), points: Vec::new() }
    }
    fn add(&mut self, p: Point) {
        self.points.push(p);
    }
    fn length(&self) -> i64 {
        let mut total = 0;
        let mut i = 1;
        while i < self.points.len() {
            total += self.points[i - 1].manhattan(&self.points[i]);
            i += 1;
        }
        total
    }
    fn first_name(&self) -> &str {
        self.name.as_str()
    }
}

fn main() {
    let mut p = Point::new(1, 2);
    p.shift(3);
    let q = p;
    println!("{:?} {:?} {}", p, q, p == q);
    let mut path = Path::new("route");
    path.add(Point::new(0, 0));
    path.add(Point { x: 3, y: 4 });
    path.add(p);
    println!("{} {}", path.first_name(), path.length());
    for pt in &mut path.points {
        pt.shift(1);
    }
    println!("{:?}", path);
}
