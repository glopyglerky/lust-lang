pub struct NodeFactory {
    pub n: i64,
}

impl NodeFactory {
    pub fn make() -> NodeFactory {
        NodeFactory { n: 1 }
    }

    pub fn id(&self) -> i64 {
        self.n
    }
}

pub fn ast_marker() -> i64 {
    40
}
