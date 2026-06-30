use crate::codegen_test_template;

codegen_test_template!(
    init_scale_without_list,
    "int a=1;
int main()
{
    int a=2;
    int b=a+1;
    return 0;
}
"
);

codegen_test_template!(
    init_scale_with_list,
    "int a={1};
int main()
{
    int a={2};
    int b={a+1};
    return 0;
}
"
);

codegen_test_template!(
    init_array_with_string,
    r#"char a[]="fdas";
char b[3][5]={"fdas"};
"#
);

codegen_test_template!(
    init_array_with_list,
    r#"int a[]={1,2,3};
int b[5]={[2]=1,2,3};
int c[4][5]={[1][2]=1,2,3};
int d[4][5]={{1,2,3},4,5};
"#
);

codegen_test_template!(
    init_struct_without_biffield,
    "struct A{
    int a;
    float b;
    double c;
}a={1,2,3};
int main()
{
    int a;
    float b;
    double c;
    struct A x={a,b,c};
    return 0;
}
"
);

codegen_test_template!(
    init_struct_with_biffield,
    "struct A{
    int a:3;
    int b:4;
    int c:5;
}a={1,2,3};
int main()
{
    int a;
    struct A b={a,a+1,a+2};
    return 0;
}
"
);

codegen_test_template!(
    init_union_without_biffield,
    "union A{
    int a;
    float b;
    double c;
}a[]={1,2,3};
int main()
{
    int a;
    float b;
    double c;
    union A x={a,b,c};
    return 0;
}
"
);

codegen_test_template!(
    init_union_with_biffield,
    "union A{
    int a:3;
    int b:4;
    int c:5;
}a[]={1,2,3};
int main()
{
    int a;
    union A b={a,a+1,a+2};
    return 0;
}
"
);
