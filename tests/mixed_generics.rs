use subenum::subenum;

#[subenum(JustRef, JustConst, JustType)]
#[derive(Debug, PartialEq, Clone)]
enum KitchenSink<'a, T, const N: usize>
where
    T: Copy + std::ops::Add<Output = T>,
{
    // Uses only lifetime
    #[subenum(JustRef)]
    RefHolder(&'a str),

    // Uses only const
    #[subenum(JustConst)]
    ArrayHolder([u8; N]),

    // Uses only Type
    #[subenum(JustType)]
    ValueHolder(T),
}

#[test]
fn test_mixed_generics_isolation() {
    const SIZE: usize = 16;
    let val: i32 = 100;

    // 1. JustRef: Should be JustRef<'a>
    // It should NOT require T or N.
    // If your macro incorrectly requires T, this might fail to compile
    // if T doesn't satisfy bounds, or simply via signature mismatch.
    let r: JustRef<'_> = JustRef::RefHolder("hello");
    match r {
        JustRef::RefHolder(s) => assert_eq!(s, "hello"),
    }

    // 2. JustConst: Should be JustConst<const N: usize>
    // Should NOT require 'a or T.
    let c: JustConst<SIZE> = JustConst::ArrayHolder([0; SIZE]);
    match c {
        JustConst::ArrayHolder(arr) => assert_eq!(arr.len(), 16),
    }

    // 3. JustType: Should be JustType<T>
    // Crucial: It MUST preserve the bounds `where T: Copy + Add...`
    // but MUST Drop the generic parameters 'a and N.
    let v: JustType<i32> = JustType::ValueHolder(val);
    match v {
        JustType::ValueHolder(x) => assert_eq!(x, 100),
    }
}
