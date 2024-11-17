use pod_derive::Fields;

#[derive(Fields, Debug, PartialEq)]
struct Data {
  pub field0: usize,
  pub field1: f32,
  pub field2: Option<usize>,
}

fn main() {
  let mut data = Data {
    field0: 42,
    field1: 0f32,
    field2: Some(84),
  };
  data.field0_mut();
  data.set_field0(*data.field0() + 1);
  data.set_field1(-1f32);
  data.set_field2(Some(12));
  assert_eq!(
    data,
    Data {
      field0: 43,
      field1: -1f32,
      field2: Some(12)
    }
  );
  println!("{:?}", data);
}
