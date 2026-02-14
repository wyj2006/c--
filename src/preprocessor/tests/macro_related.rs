use crate::preprocessor::{tests::quick_new_preprocessor, token::to_string};

#[test]
pub fn macro_example_6_10_5_1_1() {
    let mut preprocessor = quick_new_preprocessor(
        " # define LPAREN() (
#define G(Q) 42
#define F(R, X, ...) __VA_OPT__(G R X) )
int x = F(LPAREN(), 0, <:-);
#undef G
#undef F
",
    );
    let result = to_string(&preprocessor.process().unwrap());
    assert_eq!(result, " \n\n\nint x = 42 ;\n\n\n");
}

#[test]
pub fn macro_example_6_10_5_1_2_1() {
    let mut preprocessor = quick_new_preprocessor(
        "#define F(...) f(0 __VA_OPT__(,) __VA_ARGS__)
#define G(X, ...) f(0, X __VA_OPT__(,) __VA_ARGS__)
#define SDEF(sname, ...) S sname __VA_OPT__(= { __VA_ARGS__ })
#define EMP
F(a, b, c)
F()
F(EMP)
G(a, b, c)
G(a, )
G(a)
SDEF(foo);
SDEF(bar, 1, 2);
#undef F
#undef G
",
    );
    let result = to_string(&preprocessor.process().unwrap());
    assert_eq!(
        result,
        "



f(0 , a, b, c)
f(0  )
f(0  )
f(0, a ,  b, c)
f(0, a , )
f(0, a  )
S foo ;
S bar = {  1, 2 };


"
    );
}

#[test]
pub fn macro_example_6_10_5_1_2_2() {
    let mut preprocessor = quick_new_preprocessor(
        "#define H1(X, ...) X __VA_OPT__(##) __VA_ARGS__
#define H2(X, Y, ...) __VA_OPT__(X ## Y,) __VA_ARGS__
H2(a, b, c, d)
",
    );
    let result = to_string(&preprocessor.process().unwrap());
    assert_eq!(
        result,
        "

ab,  c, d
"
    );
}

#[test]
pub fn macro_example_6_10_5_1_2_3() {
    let mut preprocessor = quick_new_preprocessor(
        "#define H3(X, ...) #__VA_OPT__(X##X X##X)
H3(, 0)
#define H4(X, ...) __VA_OPT__(a X ## X) ## b
H4(, 1)
",
    );
    let result = to_string(&preprocessor.process().unwrap());
    assert_eq!(
        result,
        r#"
""

a b
"#
    );
}

#[test]
pub fn macro_example_6_10_5_1_2_4() {
    let mut preprocessor = quick_new_preprocessor(
        "#define H5A(...) __VA_OPT__()/**/__VA_OPT__()
#define H5B(X) a ## X ## b
#define H5C(X) H5B(X)
H5C(H5A())
",
    );
    let result = to_string(&preprocessor.process().unwrap());
    assert_eq!(
        result,
        "


ab
"
    );
}

#[test]
pub fn macro_example_6_10_5_3() {
    let mut preprocessor = quick_new_preprocessor(
        "#define hash_hash # ## #
#define mkstr(a) # a
#define in_between(a) mkstr(a)
#define join(c, d) in_between(c hash_hash d)
char p[] = join(x, y);
",
    );
    let result = to_string(&preprocessor.process().unwrap());
    assert_eq!(result, "\n\n\n\nchar p[] = \"x ## y\";\n");
}
