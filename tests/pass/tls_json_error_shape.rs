// The json kernel's shape: the last error per thread in a const-initialized
// RefCell<Option<T>>, set through `with` and read back with a borrow.
use std::cell::RefCell;

#[derive(Clone, Debug)]
struct SemanticError {
    go_type: String,
    offset: usize,
}

thread_local! {
    static LAST_ERROR: RefCell<Option<SemanticError>> = const { RefCell::new(None) };
}

fn remember(e: SemanticError) -> String {
    let message = format!("cannot unmarshal into {}", e.go_type);
    LAST_ERROR.with(|last| *last.borrow_mut() = Some(e));
    message
}

fn last_offset() -> Option<usize> {
    LAST_ERROR.with(|last| match &*last.borrow() {
        Some(e) => Some(e.offset),
        None => None,
    })
}

fn main() {
    println!("{:?}", last_offset());
    let m = remember(SemanticError { go_type: "int".to_string(), offset: 3 });
    println!("{} {:?}", m, last_offset());
    let m = remember(SemanticError { go_type: "string".to_string(), offset: 9 });
    println!("{} {:?}", m, last_offset());
    LAST_ERROR.with(|last| println!("{:?}", last.borrow()));
}
