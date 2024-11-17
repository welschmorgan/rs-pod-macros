use podstru_derive::Ctor;
use std::fmt::Debug;

#[derive(Ctor, Debug, PartialEq)]
struct Data {
  pub field0: usize,
  pub field1: f32,
  #[ctor(skip = "hello from 42")]
  pub field2: &'static str,
}

fn main() {
  let data = Data::new(42, 1f32);
  assert_eq!(
    data,
    Data {
      field0: 42,
      field1: 1f32,
      field2: "hello from 42",
    }
  );
  println!("{:?}", data);
}
