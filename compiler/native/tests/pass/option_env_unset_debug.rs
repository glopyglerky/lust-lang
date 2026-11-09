// An unset `option_env!` is `None::<&'static str>`, so `{:?}` of it alone type-checks.
fn main() {
    println!("{:?}", option_env!("LUST_TEST_SURELY_UNSET_VAR_7Q"));
    let v = option_env!("LUST_TEST_SURELY_UNSET_VAR_7Q");
    println!("{}", v.is_none());
}
