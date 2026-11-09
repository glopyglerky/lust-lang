// A closure that calls an impl Fn value from a fn defined LATER in the file: the build's
// discovery walk checked the user first and gave its closure a wrong kind (L0612 at the
// generic call, or wrong C env types). ts-rust tsconfig_p2.rs get_tsconfig_prop_array_element_value.
mod ast {
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct Node(pub u32);
    impl Node {
        pub const NIL: Node = Node(0);
        pub fn text(&self) -> &str { if self.0 == 2 { "t2" } else { "x" } }
        pub fn elements(self) -> Vec<Node> { (0..self.0).map(Node).collect() }
        pub fn is_some(self) -> bool { self.0 != 0 }
    }
    pub fn is_lit(n: Node) -> bool { n.0 > 0 }
}
mod ts {
    use crate::ast::*;
    pub fn get_value(tsf: Node, key: &str, element_value: &str) -> Node {
        let callback = by_value(element_value);
        for_each_prop_array(tsf, key, |property| {
            callback(property)
        })
        .unwrap_or(Node::NIL)
    }
    pub fn for_each_prop_array<T>(tsf: Node, key: &str, callback: impl FnMut(Node) -> Option<T>) -> Option<T> {
        if tsf.is_some() {
            return for_each_assign(tsf, key, callback, &[]);
        }
        None
    }
    pub fn for_each_assign<T>(obj: Node, key: &str, mut callback: impl FnMut(Node) -> Option<T>, key2: &[&str]) -> Option<T> {
        for p in obj.elements() {
            if key.is_empty() || (!key2.is_empty() && key2[0] == "q") { continue; }
            if let Some(t) = callback(p) { return Some(t); }
        }
        None
    }
    pub fn by_value(value: &str) -> impl Fn(Node) -> Option<Node> + use<> {
        let value = value.to_string();
        move |property: Node| -> Option<Node> {
            if is_lit(property) {
                return property.elements().iter().copied().find(|element| is_lit(*element) && element.text() == value);
            }
            None
        }
    }
}
fn main() {
    println!("{:?} {:?}", ts::get_value(ast::Node(4), "k", "t2"), ts::get_value(ast::Node(0), "k", "t2"));
}
