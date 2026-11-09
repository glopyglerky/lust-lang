// Statement-level `#[allow(..)]` forms: on an expression statement,
// on a block tail, on a match arm, in a match body's inner position,
// at the start of a statement block, and at the start of a fn body.
// The overflowing functions are never called; the lint is tested.
fn expr_stmt() {
    #[allow(arithmetic_overflow)]
    {
        let _x: u8 = 255u8 + 1u8;
    };
}

fn tail_allow() -> u8 {
    #[allow(arithmetic_overflow)]
    {
        let a: u8 = 255;
        a + 1
    }
}

fn arm_allow() {
    match 1 {
        #[allow(arithmetic_overflow)]
        _ => {
            let _x: u8 = 255u8 + 1u8;
        }
    }
}

fn match_inner_allow() {
    match 1 {
        #![allow(arithmetic_overflow)]
        _ => {
            let _x: u8 = 255u8 + 1u8;
        }
    }
}

fn block_inner_allow() {
    {
        #![allow(arithmetic_overflow)]
        let _x: u8 = 255u8 + 1u8;
    }
}

fn fn_inner_allow() -> u8 {
    #![allow(arithmetic_overflow)]
    let a: u8 = 255;
    a + 1
}

fn main() {
    let _ = expr_stmt;
    let _ = tail_allow;
    let _ = arm_allow;
    let _ = match_inner_allow;
    let _ = block_inner_allow;
    let _ = fn_inner_allow;
    println!("ok");
}
