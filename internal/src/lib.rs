pub trait Builder {
  type Target;

  fn builder() -> Self::Target
  where
    Self: Sized;
}
