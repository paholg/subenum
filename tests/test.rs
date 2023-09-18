use subenum::subenum;

#[subenum(EnumB, EnumC, EnumD)]
#[derive(PartialEq, Debug, Clone)]
enum EnumA<T, U> where
T: From<String>,
U: Copy {
    #[subenum(EnumC, EnumD)]
    A,
    #[subenum(EnumB, EnumC)]
    B(T),
    #[subenum(EnumB)]
    C(U)
}