# POD

[![Apache License](https://img.shields.io/badge/license-Apache%202.0-orange.svg?style=flat-square)](http://www.apache.org/licenses/LICENSE-2.0) [![Crates.io](https://img.shields.io/crates/v/podstru.svg?style=flat-square)]() [![rust](https://img.shields.io/badge/rust-1.78.0%20stable-blue.svg?style=flat-square)]() [![release](https://img.shields.io/badge/release-0.1.1-darkgreen.svg?style=flat-square)]()

The `pod` crate supports auto-generating the most basic operations on pod-like structs.
It alleviates the burden of having to write bureaucratic funcs and patterns.

It currently supports the following:

- Generating a builder pattern via the `Builder` derive macro
- Generating field setters via the `Setters` derive macro
- Generating field getters via the `Getters` derive macro
- Generating field setters *AND* getters via the `Fields` derive macro
- Generating `new` constructor via the `Ctor` derive macro

## Crates structure

3 crates are available:

- internal - this provides the traits used by the derive macros
- derive - the derive macros
- public - the reexported traits and derive macros

## Examples

- [derive/builder](derive/examples/builder.rs)
- [derive/ctor](derive/examples/ctor.rs)
- [derive/fields](derive/examples/fields.rs)
- [derive/getters](derive/examples/getters.rs)
- [derive/setters](derive/examples/setters.rs)

## Authors

- Morgan Welsch