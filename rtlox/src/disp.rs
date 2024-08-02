use std::fmt::Display;


pub fn display_vec<T: Display>(vec: &[T]) -> String {
  vec.iter()
    .map(|x| x.to_string())
    .collect::<Vec<String>>()
    .join("; ")
}

pub fn display_option<T: Display>(opt: &Option<T>) -> String {
  match opt {
    Some(inner) => inner.to_string(),
    None => "None".into()
  }
}