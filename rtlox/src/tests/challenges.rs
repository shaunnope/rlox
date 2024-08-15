use super::*;


#[test]  
fn obj_from_closure() -> Result<(), Box<dyn Error>> {
  let path = Path::new("../custom_tests").join("closures").join("class_from_closure.lox");
  println!("\n{:?}", path);
  run_file(path)?;
  
  Ok(())
}