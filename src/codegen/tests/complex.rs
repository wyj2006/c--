use crate::codegen_test_template;

codegen_test_template!(
    init_cast_op,
    "int main()
{
    float _Complex a=1;
    float _Complex b=1.2;
    float _Complex c=a+b+1i;
}
"
);
