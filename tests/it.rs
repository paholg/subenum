use subenum::subenum;

#[subenum(Bar)]
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

#[subenum(Both, Str, Tee, Neither)]
#[derive(Clone, Debug)]
enum Pippy<'a, T> {
    #[subenum(Both)]
    A(&'a str, T),
    #[subenum(Both, Str)]
    B { a: &'a str },
    #[subenum(Both, Tee)]
    C { a: T },
    #[subenum(Neither, Tee)]
    D,
    #[subenum(Neither, Str)]
    E(String),
    #[subenum(Neither, Both)]
    F { a: u32 },
    #[subenum(Str)]
    G { recursive: &'a Pippy<'a, T> },
}

#[subenum(Flip, Flop)]
#[derive(Clone, Debug, PartialEq, Eq)]
enum Flippy<T, U>
where
    T: Clone,
{
    #[subenum(Flip)]
    A(T),
    #[subenum(Flop)]
    B(U),
}

#[subenum(Floop)]
#[derive(Clone, Debug, PartialEq)]
enum Snoo<T> {
    #[subenum(Floop)]
    A(T),
}

#[test]
fn test_snoo() {
    let a: Snoo<u32> = Snoo::A(3);
    let b: Floop<u32> = a.clone().try_into().unwrap();

    assert_eq!(a, b);
}

#[subenum(Boop)]
#[derive(Clone, Debug, PartialEq, Eq)]
enum Boo<'a> {
    #[subenum(Boop)]
    A(&'a str),
}

#[subenum(Phew)]
enum Whew<'a: 'b, 'b, 'c, T, U> {
    #[subenum(Phew)]
    A(&'a T),
    #[subenum(Phew)]
    B(&'b [&'c [U; 7]]),
}

#[test]
fn excersice() {
    let _ = Bar::B {
        x: 1,
        y: 2,
        z: "yes".into(),
        w: "no".into(),
    };

    let _ = Pippy::A("hello", 7);

    let _ = Flip::A(8);

    let _ = Boop::A("beep");

    let _: Whew<'_, '_, '_, i32, u8> = Whew::A(&7);
}

#[subenum(SubEnumWithErrorVariant)]
#[derive(Debug, Clone, Copy, PartialEq)]
enum EnumWithErrorVariant {
    #[subenum(SubEnumWithErrorVariant)]
    Error,
}

#[test]
fn test_enum_with_error_variant() {
    let a = EnumWithErrorVariant::Error;
    let b = SubEnumWithErrorVariant::try_from(a).unwrap();

    assert_eq!(a, b);
}
