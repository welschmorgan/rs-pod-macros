# POD

The `pod` crate supports auto-generating the most basic operations on pod-like structs.
It alleviates the burden of having to write bureaucratic funcs and patterns.

It currently supports the following:

- Generating a builder pattern via the `Builder` derive macro
- Generating field setters via the `Setters` derive macro
- Generating field getters via the `Getters` derive macro
- Generating field setters *AND* getters via the `Fields` derive macro
- Generating `new` constructor via the `Ctor` derive macro

## Crates structure

3 craves are available:

- internal - this provides the traits used by the derive macros
- derive - the derive macros
- public - the reexported traits and derive macros

## Examples

- [builder](examples/builder.rs)
- [ctor](examples/ctor.rs)
- [fields](examples/fields.rs)
- [getters](examples/getters.rs)
- [setters](examples/setters.rs)

## Authors

- Morgan Welsch