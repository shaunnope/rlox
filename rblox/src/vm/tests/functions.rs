use super::*;

#[test]
fn can_declare_function() {
  let source = "fun areWeHavingItYet(a, b) {
  print \"Yes we are!\";
}

print areWeHavingItYet;";
  let mut vm = VM::new();

  if let Err(err) = vm.run(source) {
    eprintln!("{err:?}")
  };

}

#[test]
fn displays_stack_trace() {
  let source = "fun a() { b(); }
fun b() { c(); }
fun c() {
  c(\"too\", \"many\");
}

a();";

  let mut vm = VM::new();

  if let Err(err) = vm.run(source) {
    eprintln!("{err:?}")
  };
}

#[ignore]
#[test]
fn native_clock() {
  let source = "fun fib(n) {
  if (n < 2) return n;
  return fib(n - 2) + fib(n - 1);
}

var start = clock();
print fib(2);
print clock() - start;";

  let mut vm = VM::new();

  if let Err(err) = vm.run(source) {
    eprintln!("{err:?}")
  };
}

#[test]
fn closure_in_stack() {
  let source = "fun outer() {
  var x = \"outside\";
  fun inner() {
    print x;
  }
  inner();
}
outer();";

  let mut vm = VM::new();

  if let Err(err) = vm.run(source) {
    eprintln!("{err:?}")
  };
}

#[test]
fn closure_off_stack() {
  let source = "fun outer() {
  var x = \"outside\";
  fun inner() {
    print x;
  }
  x = \"changed\";

  return inner;
}
var x = \"global\";
var closure = outer();
closure();";

  let mut vm = VM::new();

  if let Err(err) = vm.run(source) {
    eprintln!("{err:?}")
  };
}

#[test]
fn closure_variables() {
  let source = 
"var globalSet;
var globalGet;

fun main() {
  var a = \"initial\";

  fun set() { a = \"updated\"; }
  fun get() { print a; }

  globalSet = set;
  globalGet = get;
}

main();
globalSet();
globalGet();";

  let mut vm = VM::new();

  if let Err(err) = vm.run(source) {
    eprintln!("{err:?}")
  };
}