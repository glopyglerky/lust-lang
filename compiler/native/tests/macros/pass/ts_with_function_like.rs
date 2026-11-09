// macro: with_function_like_data
// source: crates/ts_goport/src/frontend/parser/reparser.rs:31
// The macro_rules! text below is copied exact from ts-rust. Only the
// surrounding types are minimal stand-ins. See tests/macros/README.md.
include!("node_scaffold.rs.inc");

pub type D = NodeData;

macro_rules! with_function_like_data {
    ($data:expr, $d:ident => $body:expr, index $i:ident => $index_body:expr) => {
        match $data {
            D::ArrowFunction($d) => $body,
            D::CallSignatureDeclaration($d) => $body,
            D::ConstructSignatureDeclaration($d) => $body,
            D::ConstructorDeclaration($d) => $body,
            D::ConstructorTypeNode($d) => $body,
            D::FunctionDeclaration($d) => $body,
            D::FunctionExpression($d) => $body,
            D::FunctionTypeNode($d) => $body,
            D::GetAccessorDeclaration($d) => $body,
            D::JsDocSignature($d) => $body,
            D::MethodDeclaration($d) => $body,
            D::MethodSignatureDeclaration($d) => $body,
            D::SetAccessorDeclaration($d) => $body,
            D::IndexSignatureDeclaration($i) => $index_body,
            other => panic!("node data is not function-like: {}", other.schema_name()),
        }
    };
}

fn classify(data: &NodeData) -> i32 {
    with_function_like_data!(data, d => d.value + 1, index i => i.value + 100)
}

fn main() {
    let f = NodeData::FunctionDeclaration(Box::new(FunctionDeclarationData { value: 10, ..Default::default() }));
    let a = NodeData::ArrowFunction(Box::new(ArrowFunctionData { value: 20, ..Default::default() }));
    let i = NodeData::IndexSignatureDeclaration(Box::new(IndexSignatureDeclarationData { value: 3, ..Default::default() }));
    println!("fn={} arrow={} index={}", classify(&f), classify(&a), classify(&i));
    std::panic::set_hook(Box::new(|_| {}));
    let other = NodeData::Identifier(Box::default());
    let r = std::panic::catch_unwind(|| classify(&other));
    println!("other panics={}", r.is_err());
}
