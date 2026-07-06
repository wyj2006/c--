use crate::codegen_riscv_test_template;

codegen_riscv_test_template!(
    conditional,
    "int main()
{
    int a,b,c,d;
    a=b?c:d;
}
"
);

codegen_riscv_test_template!(
    subscript,
    "int main()
{
    int a[5];
    a[2]=a[3];
    int b[7][8];
    b[4][5]=b[6][7];
}"
);

codegen_riscv_test_template!(
    member_access,
    "typedef struct{
    int a;
    int b[6];
    int c:2;
    int d:4;
}X;

int main()
{
    X x;
    x.a=x.b[3];
    x.c=x.d;
}
"
);

codegen_riscv_test_template!(
    unaryop,
    "int main()
{
    int a,*b,**e;
    float c,d;
    e=&b;
    a=*b;
    a=~a;
    c=-d;
    *b=!a;
    e++;
    e--;
    --e;
    ++e;
}
"
);

codegen_riscv_test_template!(
    binop,
    "int main()
{
    int a,*b;
    float c,d;
    a=a+*b;
    b=b-4;
    c=c*d;
    a=b && c || d;
}
"
);

codegen_riscv_test_template!(
    cast,
    "int main()
{
    char a;
    short b;
    unsigned int c;
    float e;
    double f;

    a=b;
    a=c;
    b=a;
    c=a;

    a=e;
    e=c;
    f=b;
    b=f;
}"
);

codegen_riscv_test_template!(
    call,
    "int f(int n)
{
    if(n==1||n==2)return 1;
    return f(n-1)+f(n-2);
}
int main()
{
    int a,b;
    a=f(b);
}
"
);
