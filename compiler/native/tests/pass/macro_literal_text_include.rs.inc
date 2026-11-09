// Included by macro_literal_text_include.rs. The literals below sit at
// byte offsets that hold other text in the including file.
macro_rules! sx { () => { stringify!(0x10 '\x61' "\x62" 1_000 r"z" 1.50 b"\x63") } }
macro_rules! one { (1) => { "one" }; ($t:tt) => { "other" } }
