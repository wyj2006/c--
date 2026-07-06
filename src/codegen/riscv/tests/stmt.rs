use crate::codegen_riscv_test_template;

codegen_riscv_test_template!(
    if_stmt,
    "int main()
{
    int a;
    if(a) a++;
    else a--;
}
"
);

codegen_riscv_test_template!(
    loop_stmt,
    "int main()
{
    int a,b;
    for(int i=0;i<=b;i++)
    {
        if(i%2==0)break;
        if(i%3==0)continue;
        a+=i;
    }
}
"
);

codegen_riscv_test_template!(
    switch,
    "int main()
{
    int a;
    switch(a)
    {
        a=1;
        case 1:a=2;break;
        case 2:a=3;
        case 3:a=4;break;
        default:a=5;
    }
}"
);
