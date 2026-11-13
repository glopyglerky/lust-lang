// `{:#?}`: the Debug builders' pretty form, with PadAdapter indentation
// for nested values, one-field tuples, empty collections, and strings
// that hold newlines (escaped, so no extra indent).
#[derive(Debug)]
enum Tree {
    Leaf(i32),
    Node { left: Box<Tree>, right: Box<Tree>, label: String },
}

#[derive(Debug)]
struct Empty {}

#[derive(Debug)]
struct Wrap(Vec<Option<(u8, char)>>);

fn main() {
    let t = Tree::Node { left: Box::new(Tree::Leaf(1)), right: Box::new(Tree::Leaf(-2)), label: "a\nb".to_string() };
    println!("{:#?}", t);
    println!("{:#?}", (1,));
    println!("{:#?}", ());
    println!("{:#?}", Empty {});
    println!("{:#?}", Vec::<i32>::new());
    println!("{:#?}", Wrap(vec![Some((1, 'x')), None]));
    println!("{:#?}", [[1, 2], [3, 4]]);
    println!("{:#?}", Some(Ok::<&str, i32>("ok")));
    println!("{:#5?}", vec![1, 22]);
    println!("{:#x?}", (255, vec![16u8]));
    let s: &[i64] = &[-1, 0];
    println!("{:#?} {:?}", s, s);
}
