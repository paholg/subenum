[![crates.io](https://img.shields.io/crates/v/subenum.svg)](https://crates.io/crates/subenum)
[![Build Status](https://github.com/paholg/subenum/actions/workflows/check.yml/badge.svg)](https://github.com/paholg/subenum/actions/workflows/check.yml)
[![docs.rs](https://img.shields.io/docsrs/subenum)](https://docs.rs/subenum)

# subenum

Subenum is a simple proc-macro to derive subsets of enums. It allows conversion
between the parent and the child, will derive any traits on the child that you
have on the parent, and will implement `PartialEq` between the parent and child
if you derive it on the parent.

## Simple Example

I think the simplest way to explain it is with an example:

```rust
use subenum::subenum;

#[subenum]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Plant {
    #[subenum(Edible)]
    Basil,
    #[subenum(Edible)]
    Tomato,
    Manzanita,
    Pine,
}

fn main() -> Result<(), EdibleConvertError> {
    let plant = Plant::Tomato;

    // We can convert between them.
    let edible = Edible::try_from(plant)?;
    let _plant2 = Plant::from(edible);

    // We can compare them.
    assert_eq!(plant, edible);

    // We derive anything that's derived on the parent, such as clone.
    let edible2 = edible.clone();

    Ok(())
}
```

## Complex Example

In addition to simple enums and built-in traits, `subenum` works with complex enums and third-party attributes.

```rust
use subenum::subenum;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppleType {
    CosmicCrisp,
    Fuji,
    PinkLady,
}

#[subenum]
#[derive(Debug, Clone, Copy, PartialEq, strum::Display)]
pub enum Plant {
    #[subenum(Foo)]
    #[strum(serialize = "This is not a plant!")]
    Foo { x: i32, y: i32 },
    #[subenum(Tree, Edible)]
    Apple(AppleType),
    #[subenum(Grass)]
    Bamboo,
    #[subenum(Edible)]
    Basil,
    #[subenum(Tree)]
    Fir,
    #[subenum(Tree)]
    Pine,
    #[subenum(Edible)]
    Tomato,
    #[subenum(Edible, Grass)]
    Wheat,
}

fn main() -> Result<(), TreeConvertError> {
    let plant = Plant::Apple(AppleType::CosmicCrisp);
    let tree = Tree::try_from(plant)?;

    assert_eq!(plant, tree);

    let tree2 = tree.clone();
    assert_eq!(tree2.to_string(), "Apple");

    let foo = Foo::Foo { x: 3, y: 4 };
    assert_eq!(foo.to_string(), "This is not a plant!");

    let edible = Edible::Basil;
    let plant = Plant::from(edible);

    assert_eq!(plant.to_string(), "Basil");

    // Can't compare two subenums.
    // assert_ne!(tree2, edible);

    Ok(())
}
```


## Limitations

Currently, generics are not supported. If desired, please open a ticket.
