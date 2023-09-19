use subenum::subenum;

#[subenum(
    Binary(derive(derive_more::Display)),
    Unary,
    Keyword(derive(Copy, strum::EnumString), strum(serialize_all = "snake_case"))
)]
#[derive(Clone, Debug, PartialEq)]
enum Token {
    #[subenum(Binary(display(fmt = "-")), Unary)]
    Minus,
    #[subenum(Binary(display(fmt = "+")))]
    Plus,
    #[subenum(Keyword)]
    And,
    #[subenum(Keyword)]
    Or,
    #[subenum(Keyword)]
    Var,
}

#[test]
fn test_token() {
    let a = Token::Minus;
    let b = Binary::try_from(a.clone()).unwrap();
    println!("b: {}", b);

    let c = "and".parse::<Keyword>().unwrap();
    let d = Token::from(c);
    println!("{:?} {:?}", c, d);

    assert_eq!(a, b);
}

#[subenum(EnumB)]
enum EnumA<T> {
    #[subenum(EnumB)]
    B,
    #[subenum(EnumB)]
    C(T),
}
