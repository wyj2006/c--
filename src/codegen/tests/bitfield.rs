use crate::codegen_test_template;

codegen_test_template!(
    load_bitfield,
    "struct A{
    int a:3;
    int b:4;
    double d;
    int c:5;
};
int main()
{
    struct A a;
    a.a;
    a.b;
    a.c;
    return 0;
}
"
);

codegen_test_template!(
    store_bitfield,
    "struct A{
    int a:3;
    int b:4;
    double d;
    int c:5;
};
int main()
{
    struct A a;
    a.a=1;
    a.b=2;
    a.c=3;
    return 0;
}
"
);
