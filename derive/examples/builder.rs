use pod_derive::Builder;
use pod_internal::Builder;
use std::fmt::Debug;

#[derive(Builder, Debug)]
struct Data {
  pub field0: usize,
  pub field1: f32,
  #[builder(default = 42)]
  pub field2: Option<usize>,
}

fn main() {
  let data = Data::builder().with_field0(42).build();
  println!("{:?}", data);
}
