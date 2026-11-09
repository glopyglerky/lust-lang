// semantics: mixed-site hygiene for locals and labels (ARCH 6.3).
// Names written in a macro body do not capture caller locals.
// An ident the caller passes in keeps the caller's context and
// binds the caller's variable.

macro_rules! with_tmp {
    ($e:expr) => {{
        let tmp = 100;
        tmp + $e
    }};
}

// The macro assigns through the ident the caller gives it.
macro_rules! set_var {
    ($name:ident, $v:expr) => {
        $name = $v
    };
}

// The loop label is written in the macro body. A caller loop with
// the same written label name must not be the break target.
macro_rules! count_to {
    ($n:expr) => {{
        let mut acc = 0;
        'done: loop {
            acc += 1;
            if acc >= $n {
                break 'done;
            }
        }
        acc
    }};
}

// A binding the macro introduces shadows nothing outside: the
// caller's same-named local keeps its value after the call.
macro_rules! shadow_inside {
    () => {{
        let value = 1;
        value
    }};
}

fn main() {
    let tmp = 5;
    let r = with_tmp!(tmp);
    println!("tmp kept={tmp} macro tmp plus caller tmp={r}");

    let mut target = 0;
    set_var!(target, 42);
    println!("passed ident assigned={target}");

    let mut outer_rounds = 0;
    'done: loop {
        outer_rounds += 1;
        let inner = count_to!(3);
        println!("inner count={inner}");
        if outer_rounds == 2 {
            break 'done;
        }
    }
    println!("outer rounds={outer_rounds}");

    let value = 9;
    println!("shadow inside={} caller={}", shadow_inside!(), value);
}
