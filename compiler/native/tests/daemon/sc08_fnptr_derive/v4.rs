mod util {
    pub fn one() -> u32 {
        1
    }
}

#[derive(Clone, Debug)]
pub struct X {
    pub a: u32,
}

#[derive(Debug, Default)]
pub struct Other {
    pub b: u32,
}

fn key(p: &(i32, X)) -> i32 {
    p.0 + p.1.a as i32
}

pub fn externals(v: &[(i32, X)]) -> Vec<i32> {
    let f: fn(&(i32, X)) -> i32 = key;
    v.iter().map(f).collect()
}

#[allow(dead_code)]
fn pad_0(x: u32) -> u32 {
    x + 0
}

#[allow(dead_code)]
fn pad_1(x: u32) -> u32 {
    x + 1
}

#[allow(dead_code)]
fn pad_2(x: u32) -> u32 {
    x + 2
}

#[allow(dead_code)]
fn pad_3(x: u32) -> u32 {
    x + 3
}

#[allow(dead_code)]
fn pad_4(x: u32) -> u32 {
    x + 4
}

#[allow(dead_code)]
fn pad_5(x: u32) -> u32 {
    x + 5
}

#[allow(dead_code)]
fn pad_6(x: u32) -> u32 {
    x + 6
}

#[allow(dead_code)]
fn pad_7(x: u32) -> u32 {
    x + 7
}

#[allow(dead_code)]
fn pad_8(x: u32) -> u32 {
    x + 8
}

#[allow(dead_code)]
fn pad_9(x: u32) -> u32 {
    x + 9
}

#[allow(dead_code)]
fn pad_10(x: u32) -> u32 {
    x + 10
}

#[allow(dead_code)]
fn pad_11(x: u32) -> u32 {
    x + 11
}

#[allow(dead_code)]
fn pad_12(x: u32) -> u32 {
    x + 12
}

#[allow(dead_code)]
fn pad_13(x: u32) -> u32 {
    x + 13
}

#[allow(dead_code)]
fn pad_14(x: u32) -> u32 {
    x + 14
}

#[allow(dead_code)]
fn pad_15(x: u32) -> u32 {
    x + 15
}

#[allow(dead_code)]
fn pad_16(x: u32) -> u32 {
    x + 16
}

#[allow(dead_code)]
fn pad_17(x: u32) -> u32 {
    x + 17
}

#[allow(dead_code)]
fn pad_18(x: u32) -> u32 {
    x + 18
}

#[allow(dead_code)]
fn pad_19(x: u32) -> u32 {
    x + 19
}

#[allow(dead_code)]
fn pad_20(x: u32) -> u32 {
    x + 20
}

#[allow(dead_code)]
fn pad_21(x: u32) -> u32 {
    x + 21
}

#[allow(dead_code)]
fn pad_22(x: u32) -> u32 {
    x + 22
}

#[allow(dead_code)]
fn pad_23(x: u32) -> u32 {
    x + 23
}

#[allow(dead_code)]
fn pad_24(x: u32) -> u32 {
    x + 24
}

#[allow(dead_code)]
fn pad_25(x: u32) -> u32 {
    x + 25
}

#[allow(dead_code)]
fn pad_26(x: u32) -> u32 {
    x + 26
}

#[allow(dead_code)]
fn pad_27(x: u32) -> u32 {
    x + 27
}

#[allow(dead_code)]
fn pad_28(x: u32) -> u32 {
    x + 28
}

#[allow(dead_code)]
fn pad_29(x: u32) -> u32 {
    x + 29
}

#[allow(dead_code)]
fn pad_30(x: u32) -> u32 {
    x + 30
}

#[allow(dead_code)]
fn pad_31(x: u32) -> u32 {
    x + 31
}

#[allow(dead_code)]
fn pad_32(x: u32) -> u32 {
    x + 32
}

#[allow(dead_code)]
fn pad_33(x: u32) -> u32 {
    x + 33
}

#[allow(dead_code)]
fn pad_34(x: u32) -> u32 {
    x + 34
}

#[allow(dead_code)]
fn pad_35(x: u32) -> u32 {
    x + 35
}

#[allow(dead_code)]
fn pad_36(x: u32) -> u32 {
    x + 36
}

#[allow(dead_code)]
fn pad_37(x: u32) -> u32 {
    x + 37
}

#[allow(dead_code)]
fn pad_38(x: u32) -> u32 {
    x + 38
}

#[allow(dead_code)]
fn pad_39(x: u32) -> u32 {
    x + 39
}

#[allow(dead_code)]
fn pad_40(x: u32) -> u32 {
    x + 40
}

#[allow(dead_code)]
fn pad_41(x: u32) -> u32 {
    x + 41
}

#[allow(dead_code)]
fn pad_42(x: u32) -> u32 {
    x + 42
}

#[allow(dead_code)]
fn pad_43(x: u32) -> u32 {
    x + 43
}

#[allow(dead_code)]
fn pad_44(x: u32) -> u32 {
    x + 44
}

#[allow(dead_code)]
fn pad_45(x: u32) -> u32 {
    x + 45
}

#[allow(dead_code)]
fn pad_46(x: u32) -> u32 {
    x + 46
}

#[allow(dead_code)]
fn pad_47(x: u32) -> u32 {
    x + 47
}

#[allow(dead_code)]
fn pad_48(x: u32) -> u32 {
    x + 48
}

#[allow(dead_code)]
fn pad_49(x: u32) -> u32 {
    x + 49
}

#[allow(dead_code)]
fn pad_50(x: u32) -> u32 {
    x + 50
}

#[allow(dead_code)]
fn pad_51(x: u32) -> u32 {
    x + 51
}

#[allow(dead_code)]
fn pad_52(x: u32) -> u32 {
    x + 52
}

#[allow(dead_code)]
fn pad_53(x: u32) -> u32 {
    x + 53
}

#[allow(dead_code)]
fn pad_54(x: u32) -> u32 {
    x + 54
}

#[allow(dead_code)]
fn pad_55(x: u32) -> u32 {
    x + 55
}

#[allow(dead_code)]
fn pad_56(x: u32) -> u32 {
    x + 56
}

#[allow(dead_code)]
fn pad_57(x: u32) -> u32 {
    x + 57
}

#[allow(dead_code)]
fn pad_58(x: u32) -> u32 {
    x + 58
}

#[allow(dead_code)]
fn pad_59(x: u32) -> u32 {
    x + 59
}

#[allow(dead_code)]
fn pad_60(x: u32) -> u32 {
    x + 60
}

#[allow(dead_code)]
fn pad_61(x: u32) -> u32 {
    x + 61
}

#[allow(dead_code)]
fn pad_62(x: u32) -> u32 {
    x + 62
}

#[allow(dead_code)]
fn pad_63(x: u32) -> u32 {
    x + 63
}

#[allow(dead_code)]
fn pad_64(x: u32) -> u32 {
    x + 64
}

#[allow(dead_code)]
fn pad_65(x: u32) -> u32 {
    x + 65
}

#[allow(dead_code)]
fn pad_66(x: u32) -> u32 {
    x + 66
}

#[allow(dead_code)]
fn pad_67(x: u32) -> u32 {
    x + 67
}

#[allow(dead_code)]
fn pad_68(x: u32) -> u32 {
    x + 68
}

#[allow(dead_code)]
fn pad_69(x: u32) -> u32 {
    x + 69
}

#[allow(dead_code)]
fn pad_70(x: u32) -> u32 {
    x + 70
}

#[allow(dead_code)]
fn pad_71(x: u32) -> u32 {
    x + 71
}

#[allow(dead_code)]
fn pad_72(x: u32) -> u32 {
    x + 72
}

#[allow(dead_code)]
fn pad_73(x: u32) -> u32 {
    x + 73
}

#[allow(dead_code)]
fn pad_74(x: u32) -> u32 {
    x + 74
}

#[allow(dead_code)]
fn pad_75(x: u32) -> u32 {
    x + 75
}

#[allow(dead_code)]
fn pad_76(x: u32) -> u32 {
    x + 76
}

#[allow(dead_code)]
fn pad_77(x: u32) -> u32 {
    x + 77
}

#[allow(dead_code)]
fn pad_78(x: u32) -> u32 {
    x + 78
}

#[allow(dead_code)]
fn pad_79(x: u32) -> u32 {
    x + 79
}

#[allow(dead_code)]
fn pad_80(x: u32) -> u32 {
    x + 80
}

#[allow(dead_code)]
fn pad_81(x: u32) -> u32 {
    x + 81
}

#[allow(dead_code)]
fn pad_82(x: u32) -> u32 {
    x + 82
}

#[allow(dead_code)]
fn pad_83(x: u32) -> u32 {
    x + 83
}

#[allow(dead_code)]
fn pad_84(x: u32) -> u32 {
    x + 84
}

#[allow(dead_code)]
fn pad_85(x: u32) -> u32 {
    x + 85
}

#[allow(dead_code)]
fn pad_86(x: u32) -> u32 {
    x + 86
}

#[allow(dead_code)]
fn pad_87(x: u32) -> u32 {
    x + 87
}

#[allow(dead_code)]
fn pad_88(x: u32) -> u32 {
    x + 88
}

#[allow(dead_code)]
fn pad_89(x: u32) -> u32 {
    x + 89
}

#[allow(dead_code)]
fn pad_90(x: u32) -> u32 {
    x + 90
}

#[allow(dead_code)]
fn pad_91(x: u32) -> u32 {
    x + 91
}

#[allow(dead_code)]
fn pad_92(x: u32) -> u32 {
    x + 92
}

#[allow(dead_code)]
fn pad_93(x: u32) -> u32 {
    x + 93
}

#[allow(dead_code)]
fn pad_94(x: u32) -> u32 {
    x + 94
}

#[allow(dead_code)]
fn pad_95(x: u32) -> u32 {
    x + 95
}

#[allow(dead_code)]
fn pad_96(x: u32) -> u32 {
    x + 96
}

#[allow(dead_code)]
fn pad_97(x: u32) -> u32 {
    x + 97
}

#[allow(dead_code)]
fn pad_98(x: u32) -> u32 {
    x + 98
}

#[allow(dead_code)]
fn pad_99(x: u32) -> u32 {
    x + 99
}

pub fn ident<T: Copy>(t: T) -> T {
    t
}

fn main() {
    let v = vec![(1, X { a: 2 }), (3, X { a: 4 })];
    let o = Other { b: 5 };
    println!("{:?} {} {} {}", externals(&v), o.b, ident(1u8), util::one());
}
