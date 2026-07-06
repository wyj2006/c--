use crate::codegen_riscv_test_template;

codegen_riscv_test_template!(
    global,
    r#"typedef struct{
    long long a;
    int b;
    char c;
}X;
X x={1,2,3};
X y;
char z[]="fdas";
"#
);

codegen_riscv_test_template!(
    local,
    "int main()
{
    int a[5]={[2]=3,4};
    int b;
    b=(int){1};
}
"
);
