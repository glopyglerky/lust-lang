#[test]
fn nested_private_module() {
    assert_eq!(core::mem::size_of::<[u16; 3]>(), 6);
}
