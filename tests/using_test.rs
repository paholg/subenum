use subenum::subenum;

// 1. Define a dummy "Child" type that wraps an integer
// This represents the "Complex" type in the parent
#[derive(Debug, PartialEq, Clone)]
pub struct BigIntWrapper(u64);

// 2. Define a simpler type for the SubEnum
// This represents the "Nested" type you want to swap in
#[derive(Debug, PartialEq, Clone)]
pub struct SmallIntWrapper(u32);

// Essential: Implement From so the conversion works automatically
impl From<SmallIntWrapper> for BigIntWrapper {
    fn from(s: SmallIntWrapper) -> Self {
        BigIntWrapper(s.0 as u64)
    }
}

impl PartialEq<SmallIntWrapper> for BigIntWrapper {
    fn eq(&self, other: &SmallIntWrapper) -> bool {
        self.0 == other.0 as u64
    }
}

impl PartialEq<BigIntWrapper> for SmallIntWrapper {
    fn eq(&self, other: &BigIntWrapper) -> bool {
        self.0 as u64 == other.0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ConversionError;

impl TryFrom<BigIntWrapper> for SmallIntWrapper {
    type Error = ConversionError;

    fn try_from(b: BigIntWrapper) -> Result<Self, Self::Error> {
        // Check for overflow: u64 value must fit inside a u32 (max 4,294,967,295)
        if b.0 > u32::MAX as u64 {
            return Err(ConversionError);
        }

        // Safety: If we reach here, the value fits.
        Ok(SmallIntWrapper(b.0 as u32))
    }
}

// 3. Define the Main Enum with the new syntax
#[subenum(MiniEnum)]
#[derive(Debug, PartialEq, Clone)]
pub enum BigEnum {
    // A variant where we keep the original type
    #[subenum(MiniEnum)]
    A(u32),

    // A variant where we SWAP the type using your new feature
    // Original: BigIntWrapper
    // SubEnum:  SmallIntWrapper
    #[subenum(MiniEnum(with(SmallIntWrapper)))]
    B(BigIntWrapper),

    // A variant excluded from the subenum
    C(String),
}

#[test]
fn test_nested_type_replacement() {
    // A. Construct the SubEnum with the SMALL wrapper
    let mini = MiniEnum::B(SmallIntWrapper(42));

    // B. Convert it to the Main Enum
    // This triggers the generated From impl, which should call
    // SmallIntWrapper -> BigIntWrapper
    let big: BigEnum = mini.into();

    // C. Verify the type was converted correctly
    let expected = BigEnum::B(BigIntWrapper(42));
    assert_eq!(big, expected);
}

#[test]
fn test_field_types_in_generated_struct() {
    // This test simply fails to compile if the types are wrong.
    // We are proving that MiniEnum::B holds a SmallIntWrapper, not Big.

    let val = MiniEnum::B(SmallIntWrapper(100));

    if let MiniEnum::B(inner) = val {
        // This assertion proves 'inner' is SmallIntWrapper.
        // If it were BigIntWrapper, we couldn't access .0 as a u32 (or type inference would differ).
        let check: u32 = inner.0;
        assert_eq!(check, 100);
    } else {
        panic!("Wrong variant");
    }
}
