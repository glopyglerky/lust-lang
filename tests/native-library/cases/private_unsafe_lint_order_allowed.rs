#[deny(unsafe_code)]
#[allow(unsafe_code)]
unsafe fn helper(value: i32) -> i32 {
    value + 1
}

pub fn exposed(value: i32) -> i32 {
    unsafe { helper(value) }
}
