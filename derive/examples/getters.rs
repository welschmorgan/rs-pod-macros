use podstru_derive::Getters;

#[derive(Getters, Debug)]
struct Data {
  pub field0: usize,
  pub field1: f32,
  pub field2: Option<usize>,
  #[getters(skip)]
  pub field3: (),
}

fn main() {
  let data = Data {
    field0: 42,
    field1: 0f32,
    field2: Some(84),
    field3: (),
  };
  assert_eq!(data.field0(), &42usize);
  assert_eq!(data.field1(), &0f32);
  assert_eq!(data.field2(), Some(&84usize));
  println!("{:?}", data);
}
