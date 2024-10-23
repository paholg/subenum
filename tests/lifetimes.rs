use subenum::subenum;

#[derive(Debug, PartialEq)]
pub struct SomeCommand<'a> {
    // blueprint struct
    pub identifier: &'a str,
}

#[derive(Debug, PartialEq)]
pub struct SomeOtherCommand<'a> {
    // blueprint struct
    pub identifier: &'a str,
}

#[subenum(Command, Literal, Weird)]
#[derive(Debug, PartialEq)]
pub enum Expression<'a, 'b, 'c> {
    // enum
    #[subenum(Command)]
    CommandWithNoArgs(&'b SomeCommand<'a>),
    #[subenum(Command)]
    CommandWith1Arg(&'b SomeOtherCommand<'a>),
    #[subenum(Literal)]
    Lit(&'b str),
    #[subenum(Weird)]
    SomeThing {
        x: SomeCommand<'c>,
        y: Box<Expression<'a, 'b, 'b>>,
    },
}

#[test]
fn test_expression() {
    let a = SomeCommand {
        identifier: "hello",
    };
    let b = SomeOtherCommand {
        identifier: "world",
    };

    let d = Expression::CommandWithNoArgs(&a);
    let e = Expression::CommandWith1Arg(&b);

    assert_eq!(d, Command::CommandWithNoArgs(&a));
    assert_eq!(e, Command::CommandWith1Arg(&b));
}
