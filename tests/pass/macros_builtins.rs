// Builtin macros expanded at token level: matches!, concat!,
// stringify!, cfg!, line!/file!, option_env!, include_str!,
// include!, debug_assert!, and nested builtin use in concat!.

include!("macros_included.rs.inc");

fn main() {
    let v = 42;
    println!("matches={} {}", matches!(v, 42), matches!(v, 1));
    let some = Some(3);
    println!("matches guard={}", matches!(some, Some(n) if n > 1));

    println!("concat={}", concat!("a", 1, 'b', true));
    println!("stringify={}", stringify!(some tokens here));
    println!("stringify expr=[{}]", stringify!(1 + 2));

    println!("cfg any={} all={} not={}", cfg!(any()), cfg!(all()), cfg!(not(any())));
    println!("cfg unix={} windows={}", cfg!(unix), cfg!(windows));

    let first = line!();
    let second = line!();
    println!("line delta={} col>0={} file ends={}", second - first, column!() > 0, file!().ends_with("macros_builtins.rs"));

    println!("env missing none={}", option_env!("LUST_DIFFTEST_NOT_SET_12345").is_none());

    let text: &str = include_str!("macro_data.txt");
    println!("include_str='{}' len={} starts={}", text.trim(), text.trim().len(), text.starts_with("hello"));

    println!("included={} {}", included_answer(), included_num());

    debug_assert!(v == 42);
    debug_assert_eq!(v + 1, 43);
    debug_assert_ne!(v, 0);
    println!("debug asserts held");
}
