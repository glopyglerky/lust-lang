// semantics: cfg stripping runs before expansion (ARCH 6.1), and
// the file and env builtin macros (ARCH 6.4).
// A disabled compile_error! must not fire and a disabled include!
// of a missing file must not be read. The runner exports
// LUST_MACRO_TEST_VALUE=hello42 at compile time for env!.

#[cfg(any())]
fn never_compiled() {
    compile_error!("this cfg is off: must not fire");
}

#[cfg(any())]
mod never_included {
    include!("missing-file-that-does-not-exist.rs");
}

include!("sem_included.rs.inc");

fn main() {
    println!("cfg any={} all={}", cfg!(any()), cfg!(all()));
    println!("included fn={} const={}", included_answer(), INCLUDED_TEXT);
    let bytes: &[u8] = include_bytes!("../data/loc/en.json");
    let text: &str = include_str!("../data/loc/en.json");
    println!("include_str='{}' bytes_eq={}", text.trim(), bytes == text.as_bytes());
    println!("env={}", env!("LUST_MACRO_TEST_VALUE"));
    println!("option_env={:?} missing={:?}", option_env!("LUST_MACRO_TEST_VALUE"), option_env!("LUST_MACRO_NOT_SET_XYZ"));
    println!("concat={}", concat!("a", 1, 'b'));
    println!("stringify={}", stringify!(some tokens here));
    let first = line!();
    let second = line!();
    println!("line delta={} column>0={} file ends={} module={}",
        second - first, column!() > 0, file!().ends_with("sem_cfg_builtins.rs"), module_path!());
}
