use subenum::subenum;

// Test tuple variants, struct variants, and repeated types.
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

// Test lifetimes, generics, as well as unused lifetimes/generics.
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
}

// Test where clause
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

// Test conversion.
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

// Test lifetime relationships.
#[subenum(Dop, Mop)]
enum Dip<'a: 'b, 'b, 'c, T, U> {
    #[subenum(Dop, Mop)]
    A(&'a T),
    #[subenum(Dop)]
    B(&'b [&'c [U; 7]]),
}

// Test generic consts.
#[subenum(Phew)]
enum Whew<const N: usize> {
    #[subenum(Phew)]
    A,
    #[subenum(Phew)]
    B([u8; N]),
}
