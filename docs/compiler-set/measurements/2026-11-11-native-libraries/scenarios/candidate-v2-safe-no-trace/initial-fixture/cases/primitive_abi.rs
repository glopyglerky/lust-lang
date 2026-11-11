fn main() {
    let () = left::unit_result();
    assert_eq!(left::unit_argument((), 41), 41);
    assert!(left::bool_roundtrip(true));
    assert!(!left::bool_roundtrip(false));
    assert_eq!(left::char_roundtrip('λ'), 'λ');
    assert_eq!(left::char_roundtrip('\u{10ffff}'), '\u{10ffff}');
    assert_eq!(left::f32_roundtrip(1.25f32), 1.25f32);
    assert_eq!(left::f32_roundtrip(-16777216.0f32), -16777216.0f32);
    assert_eq!(left::f64_roundtrip(-1.125f64), -1.125f64);
    assert_eq!(left::f64_roundtrip(9007199254740992.0f64), 9007199254740992.0f64);
    assert_eq!(left::signed_roundtrip(-170141183460469231731687303715884105728i128), -170141183460469231731687303715884105728i128);
    assert_eq!(left::signed_roundtrip(170141183460469231731687303715884105727i128), 170141183460469231731687303715884105727i128);
    assert_eq!(left::unsigned_roundtrip(340282366920938463463374607431768211455u128), 340282366920938463463374607431768211455u128);
    let indirect: fn(i32, i32) -> i32 = left::combine;
    assert_eq!(indirect(10, 1), 31);
    println!("primitive ABI passed");
}
