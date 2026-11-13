// Structs and enums inside a function body (R18): block-scoped
// names, shadowing, generics of their own, and type_name paths
// through the enclosing fn, as rustc prints them.
mod m {
    pub fn get() -> i32 {
        #[derive(Debug, Clone, PartialEq)]
        struct Local { x: i32, s: String }
        #[derive(Debug)]
        enum E { A(i32), B }
        let l = Local { x: 21, s: "hi".to_string() };
        let v: Vec<Local> = vec![l.clone(), Local { x: 1, s: String::new() }];
        println!("{:?} {:?} {:?} {}", E::A(3), E::B, v, l == v[0]);
        println!("{}", std::any::type_name::<Local>());
        println!("{}", std::any::type_name::<E>());
        {
            struct Local { y: u32 }
            let l2 = Local { y: 5 };
            println!("{} {}", l2.y, std::any::type_name::<Local>());
        }
        struct Pair<T> { a: T, b: T }
        struct Node { next: Option<Box<Node>>, v: i32 }
        let p = Pair { a: 1, b: 2 };
        let n = Node { next: Some(Box::new(Node { next: None, v: 9 })), v: 1 };
        println!("{} {}", p.a + p.b, n.next.as_ref().unwrap().v);
        l.x
    }
}
fn main() { println!("{}", m::get()); }
