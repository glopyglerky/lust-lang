// expect: L0805
// rustc: local ambiguity: multiple parsing options: built-in NTs pat ('v5') or tt ('v7').
// Two matcher paths enter the same `{...}` input group; the ambiguity check must see both
// (it used to depend on whether the group's copy reused a freed address).
macro_rules! m {
    (# $v1:tt $v2:expr => 'l) => { () };
    ($v3:ident $v4:ident $({$v5:pat , $v6:stmt ,} 'l)+ {$v7:tt $v8:literal}) => { () };
    ([$(b (@) [$v9:pat_param ,])? $(=> $v10:tt),* $v11:ident] $v12:ty : => >) => { () };
}
fn main() {
    m!(x a {x @ 1 , f(x) ,} 'l {(a, b) , fn h() {} ,} 'l {(a, b) , a ,} 'l {(a, b) , let a = 1 ,} 'l {>> -1});
}
