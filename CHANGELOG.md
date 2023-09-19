# Changelog

This project follows semantic versioning.

### Unreleased
- [added] Default feature `std` and support for no-std.
- [added] Support for subenum-specific proc-macros.

### 1.0.1 (2023-02-25)
- [fixed] References to generic types.

### 1.0.0 (2023-02-20)
- [fixed] Bug when repeating a type in an unnamed variant.
- [changed] **BREAKING** All enums to be created must now be declared at the
  top-level subenum attribute.
- [added] Support for lifetimes and generics.

### 0.1.0 (2023-02-13)
- [added] Initial `subenum` macro creation.
