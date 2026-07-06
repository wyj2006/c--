use crate::codegen_riscv_test_template;

codegen_riscv_test_template!(
    global_and_locals,
    "int e=1;
int b;
int main()
{
    int a=1;
    int b;
    float c;
    double d;
    static long long e;
}
"
);

codegen_riscv_test_template!(
    function_decl,
    "void a(){}
int b(){}
float c(char n){return n;}
double d(int x,int y){}
"
);
