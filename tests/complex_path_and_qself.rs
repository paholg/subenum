use subenum::subenum;

trait Converter {
    type Type;
}

struct Wrapper;
impl Converter for Wrapper {
    type Type = i32;
}

#[subenum(Qualified)]
enum ComplexPath<C: Converter> {
    #[subenum(Qualified)]
    // This is a TypePath with a QSelf (Qualified Self)
    // The visitor must look inside the QSelf to find 'C'
    Explicit(<C as Converter>::Type),
}

#[test]
fn test_qself_visitation() {
    let c: Qualified<Wrapper> = Qualified::Explicit(100);

    match c {
        Qualified::Explicit(i) => assert_eq!(i, 100),
    }
}
