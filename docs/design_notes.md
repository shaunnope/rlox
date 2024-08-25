# Design Inspirations

## From `rs-lox/tree-lox`

- Define custom iterator for scanning and parsing
- Scanner as a sub-module of parser
- Emit additional tokens like comment, errors etc.
  - Partial error-production rules

- Token spans instead of line no.
- Separate Parse and Runtime errors

- macros for operator parsing / evaluation

- Represent expression as explicit enum `LoxValue` instead of `dyn Any`.

- extract runner functions (user-facing) into user module. 