#[derive(Debug)]
pub struct Deep(pub i64);

#[derive(Debug)]
pub enum Kind {
    A,
    B(i64),
}
