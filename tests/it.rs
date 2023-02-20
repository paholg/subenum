use subenum::subenum;

#[subenum]
#[derive(Clone, Debug, PartialEq, Eq)]
enum Foo {
    #[subenum(Bar)]
    A(String, String, String, i32, i32, i32, u32, i32),
    #[subenum(Bar)]
    B {
        x: i32,
        y: i32,
        z: String,
        w: String,
    },
}

#[test]
fn test() {}
