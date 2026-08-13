use crate::legalizer_riscv_test_template;

legalizer_riscv_test_template!(
    arithmetic,
    "int main()
{
    float _Complex a,b,c,d,e,f,g;
    a=b+c-d*e/f + -g;
}
"
);

legalizer_riscv_test_template!(
    compare,
    "int main()
{
    float _Complex a,b,c,d,e;
    e=(a==b)&&(c!=d);
}
"
);

legalizer_riscv_test_template!(
    cast,
    "int main()
{
    float a;
    float _Complex b;
    a=b;
    b=a;
    b=1;
}
"
);
