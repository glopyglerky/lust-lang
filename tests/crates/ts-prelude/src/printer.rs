pub struct NodeFactory {
    pub n: i64,
}

impl NodeFactory {
    pub fn make() -> NodeFactory {
        NodeFactory { n: 2 }
    }
}
