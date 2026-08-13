use crate::legalizer_riscv_test_template;

legalizer_riscv_test_template!(
    arithmetic,
    "int main()
{
    _BitInt(100) a,b,c,d,e,f;
    a=b+c-d + -e;
    a=f>>5;
    a++;
    a--;
    ++a;
    --a;
}
"
);

legalizer_riscv_test_template!(
    bit_op,
    "int main()
{
    _BitInt(100) a,b,c,d,e,f;
    a=b&c|d^e;
    a=~f;
}
"
);

legalizer_riscv_test_template!(
    compare,
    "int main()
{
    bool a;
    _BitInt(100) b,c,d,e,f,g;
    a=b>c;
    a=d<e;
    a=f==g;
}
"
);

legalizer_riscv_test_template!(
    cast,
    "int main()
{
    _BitInt(100) a;
    _BitInt(200) b;
    int c;
    bool d;
    float e;
    
    a=b;
    b=a;

    a=c;
    c=a;

    d=a;

    e=a;
    a=e;
}
"
);
