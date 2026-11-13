// Record fields break consistently; the closing brace returns one indent.
// panic: assertion failed:
struct Coordinates {
    horizontal_component: u64,
    vertical_component: u64,
    precision: u8,
}
impl Coordinates {
    fn finite(self) -> bool {
        self.horizontal_component < u64::MAX && self.vertical_component < u64::MAX && self.precision > 0
    }
}
fn main() {
    let base = Coordinates { horizontal_component: 1, vertical_component: 2, precision: 3 };
    assert!(Coordinates { horizontal_component: u64::MAX, vertical_component: u64::MAX, ..base }.finite());
}
