use subenum::subenum;

#[subenum(
    Storage(
        doc = "[WebAssembly storage types](\
            https://webassembly.github.io/spec/core/syntax/types.html#syntax-storagetype)\n",
        derive(strum::EnumDiscriminants),
        strum_discriminants(name(StorageType), derive(strum::FromRepr, PartialOrd, Ord, Hash))
    ),
    Val(
        doc = "[WebAssembly value types](\
            https://webassembly.github.io/spec/core/syntax/types.html#syntax-valtype)\n",
        derive(strum::EnumDiscriminants),
        strum_discriminants(name(ValType), derive(strum::FromRepr, PartialOrd, Ord, Hash))
    ),
    Num(
        doc = "[WebAssembly number types](\
            https://webassembly.github.io/spec/core/syntax/types.html#syntax-numtype)\n",
        derive(strum::EnumDiscriminants),
        strum_discriminants(name(NumType), derive(strum::FromRepr, PartialOrd, Ord, Hash))
    ),
    Int(
        doc = "[WebAssembly integer types](\
            https://webassembly.github.io/spec/core/syntax/types.html#syntax-inttype)\n",
        derive(strum::EnumDiscriminants),
        strum_discriminants(name(IntType), derive(strum::FromRepr, PartialOrd, Ord, Hash))
    ),
    Float(
        doc = "[WebAssembly float types](\
            https://webassembly.github.io/spec/core/syntax/types.html#syntax-floattype)\n",
        derive(strum::EnumDiscriminants),
        strum_discriminants(name(FloatType), derive(strum::FromRepr, PartialOrd, Ord, Hash))
    ),
    Pack(
        doc = "[WebAssembly pack types](\
            https://webassembly.github.io/spec/core/syntax/types.html#syntax-packtype)\n",
        derive(strum::EnumDiscriminants),
        strum_discriminants(name(PackType), derive(strum::FromRepr, PartialOrd, Ord, Hash))
    )
)]
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Storage<R = usize> {
    #[subenum(
        Storage(doc = "16-bit integer type (Storage)\n"),
        Pack(doc = "16-bit integer type (Pack)\n")
    )]
    I16(i16) = 0x77,

    #[subenum(
        Storage(doc = "8-bit integer type (Storage)\n"),
        Pack(doc = "8-bit integer type (Pack)\n")
    )]
    I8(i8) = 0x78,

    #[subenum(
        Storage(doc = "128-bit vector type (Storage)\n"),
        Val(doc = "128-bit vector type (Val)\n")
    )]
    V128([u8; 16]) = 0x7B,

    #[subenum(
        Storage(doc = "64-bit float type (Storage)\n"),
        Val(doc = "64-bit float type (Val)\n"),
        Num(doc = "64-bit float type (Num)\n"),
        Float(doc = "64-bit float type (Float)\n")
    )]
    F64(f64) = 0x7C,

    #[subenum(
        Storage(doc = "32-bit float type (Storage)\n"),
        Val(doc = "32-bit float type (Val)\n"),
        Num(doc = "32-bit float type (Num)\n"),
        Float(doc = "32-bit float type (Float)\n")
    )]
    F32(f32) = 0x7D,

    #[subenum(
        Storage(doc = "64-bit integer type (Storage)\n"),
        Val(doc = "64-bit integer type (Val)\n"),
        Num(doc = "64-bit integer type (Num)\n"),
        Int(doc = "64-bit integer type (Int)\n")
    )]
    I64(i64) = 0x7E,

    #[subenum(
        Storage(doc = "32-bit integer type (Storage)\n"),
        Val(doc = "32-bit integer type (Val)\n"),
        Num(doc = "32-bit integer type (Num)\n"),
        Int(doc = "32-bit integer type (Int)\n")
    )]
    I32(i32) = 0x7F,

    #[subenum(
        Storage(doc = "Reference type (Storage)\n"),
        Val(doc = "Reference type (Val)\n")
    )]
    Ref(R),
}

type _AllTypesPresent = (
    Storage,
    StorageType,
    Val,
    ValType,
    Num,
    NumType,
    Int,
    IntType,
    Float,
    FloatType,
    Pack,
    PackType,
);
