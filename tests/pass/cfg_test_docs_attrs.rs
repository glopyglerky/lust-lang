// Doc comments and attributes of every form inside cfg-disabled code.
// rustc parses disabled code but checks only the attribute grammar
// there, so all of these must pass.
fn main() {
    println!("ok");
}

#[cfg(test)]
extern "C" fn top_ffi<T, U>(a: (T, U)) -> Result<(), u8> {
    Ok(())
}

#[doc = "outer"]
#[::tool_attr]
#[unsafe(no_mangle)]
#[cfg(test)]
mod tests {
    //! Inner line doc.
    /*! Inner block doc. */
    #![allow(dead_code)]
    #![cfg_attr(test, allow(unused))]
    use super::*;

    /// A doc comment on a struct.
    /** A block doc too. */
    #[derive(Debug, Clone)]
    struct N(/// field doc
        usize);

    /// Doc on an enum.
    enum E {
        /// Variant doc.
        #[serde(rename = "a")]
        A { /** field */ x: u8 },
        B,
    }

    /// Doc on a trait.
    trait Tr {
        //! Inner trait doc.
        /// Method doc.
        fn m(&self);
        extern "C" fn c();
        /// Assoc type doc.
        type Out;
    }

    /// Doc on an impl.
    impl crate::a::Tr for N {
        /// Method doc.
        #[inline(always)]
        fn m(&self) {}
        unsafe extern "C" fn c() {}
    }

    impl super::Tr for ::std::string::String {}

    /// A test.
    #[test]
    #[ignore = "slow"]
    #[should_panic(expected = "boom")]
    #[rustfmt::skip]
    #[clippy::cognitive_complexity = "100"]
    #[tokio::test]
    #[test_case(1, 2 ; "name")]
    #[my_attr[a b c]]
    #[my_attr{a: b}]
    #[my_attr = 1 + 2]
    #[my_attr = -1]
    #[my_attr = b"x"]
    #[my_attr = concat!("a", "b")]
    #[self::my_attr]
    #[r#my_attr]
    fn it_works() {
        //! Inner doc in a fn body.
        #![allow(unused)]
        /// Doc on a statement.
        let x = 1;
        #[rustfmt::skip]
        let v = vec![x; 3];
        match x {
            /// Doc on an arm.
            #[allow(unused)]
            1 => {}
            _ => {}
        }
        assert_eq!(v.len(), 3, "msg {}", 1);
        /// Doc and attributes on expressions.
        #[allow(unused)]
        x + 1;
        let y = #[allow(unused)] 1;
        foo(#[a] 1, #[b] 2);
        // A `{}` macro call ends its statement.
        my_macro! { anything ;; }
        [1, 2];
        my_macro! {}
        |a: i32| a * 2;
        my_macro! {}
        &raw const x;
        let z = my_macro! {}.len();
    }

    /// A disabled `extern "C"` fn with commas in its generics and return type.
    #[cfg_attr(miri, ignore)]
    extern "C" fn ffi<T, U>() -> Result<(), Box<dyn std::error::Error>> {}

    /// Doc on a macro.
    macro_rules! m {
        () => {};
    }

    /// Doc on a macro call.
    thread_local! { static T: u8 = 0; }

    /// Doc on a nested module.
    mod inner {
        //! Inner doc.
        /// Doc on an extern block.
        extern "C" {
            /// Foreign doc.
            fn ext(x: i32, ...) -> i32;
        }
        /// Doc on a union.
        union U { a: u32, /// field doc
            b: f32 }
    }
}
