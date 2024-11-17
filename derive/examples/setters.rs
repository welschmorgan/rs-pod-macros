use podstru_derive::Setters;

#[derive(Setters, Debug, PartialEq)]
struct Data {
  pub field0: usize,
  pub field1: f32,
  pub field2: Option<usize>,
  #[setters(skip)]
  pub field3: (),
}

fn main() {
  let mut data = Data {
    field0: 42,
    field1: 0f32,
    field2: Some(84),
    field3: (),
  };
  data.set_field0(33);
  data.set_field1(-1f32);
  data.set_field2(Some(12));
  assert_eq!(
    data,
    Data {
      field0: 33,
      field1: -1f32,
      field2: Some(12),
      field3: ()
    }
  );
  println!("{:?}", data);
}
