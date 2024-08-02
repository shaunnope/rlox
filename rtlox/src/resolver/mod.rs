use std::{
  collections::{hash_map::Entry, HashMap}, mem
};

use crate::{
  ast::{
    expr::{self, Expr},
    stmt::{self, ClassDecl, Stmt},
  },
  data::LoxIdent,
  interpreter::Interpreter,
  resolver::error::{ErrorType, ResolveError},
  span::Span,
};

pub mod error;

#[derive(Debug)]
pub struct Resolver<'i> {
  interpreter: &'i mut Interpreter,
  state: ResolverState,
  scopes: Vec<HashMap<String, BindingState>>,
  errors: Vec<ResolveError>,
}

impl Resolver<'_> {
  pub fn resolve(mut self, stmts: &[Stmt]) -> (bool, Vec<ResolveError>) {
    self.resolve_stmts(stmts);
    (self.errors.is_empty(), self.errors)
  }

  fn resolve_stmts(&mut self, stmts: &[Stmt]) {
    for stmt in stmts {
      self.resolve_stmt(stmt);
    }
  }

  fn resolve_stmt(&mut self, stmt: &Stmt) {
    use Stmt::*;
    match &stmt {
      VarDecl(var) => {
        self.declare(&var.name);
        if let Some(init) = &var.init {
          self.resolve_expr(init);
        }
        self.define(&var.name);
      }
      FunDecl(fun) => {
        self.declare(&fun.name);
        self.define(&fun.name);

        self.resolve_fun(fun, FunctionState::Function);
      }
      ClassDecl(class) => self.resolve_class(class),
      Return(stmt) => {
        match (self.state.function, &stmt.value) {
          (FunctionState::None, _) => {
            self.error(ErrorType::Error, stmt.return_span, "Illegal return statement");
          }
          (FunctionState::Init, Some(expr::Expr::This(_))) => {},
          (FunctionState::Init, Some(_)) => {
            self.error(
              ErrorType::Warning, stmt.return_span, 
              "Initializer returns a value that is not `this`"
            );
          }
          _ => {}
        }

        if let Some(val) = &stmt.value {
          self.resolve_expr(val);
        }
      }
      If(if_stmt) => {
        self.resolve_expr(&if_stmt.cond);
        self.resolve_stmt(&if_stmt.then_branch);
        if let Some(br) = &if_stmt.else_branch {
          self.resolve_stmt(br);
        };
      }
      While(while_stmt) => {
        self.resolve_expr(&while_stmt.cond);
        self.resolve_stmt(&while_stmt.body);
      }
      Block(block) => self.scoped(|this| this.resolve_stmts(&block.stmts)),
      Expr(expr) => self.resolve_expr(&expr.expr),
      Print(print) => self.resolve_expr(&print.expr),
      Dummy(_) => unreachable!()
    };
  }

  fn resolve_class(&mut self, class: &stmt::ClassDecl) {
    let old_class_state = mem::replace(&mut self.state.class, ClassState::Class);

    self.declare(&class.name);
    self.define(&class.name);

    if let Some(super_name) = &class.super_name {
      if class.name.name == super_name.name {
        self.error(
          ErrorType::Error, super_name.span, 
          "A class cannot inherit itself"
        );
      }

      self.state.class = ClassState::SubClass;
      self.resolve_binding(super_name);

      // init a new scope with `super` defined
      // for subclass to access superclass methods
      self.begin_scope();
      self.initialize("super")

    }

    self.scoped(|this| {
      this.initialize("this");
      for method in &class.methods {
        let state = if method.name.name == "init" {
          FunctionState::Init
        } else {
          FunctionState::Method
        };
        this.resolve_fun(&method, state);
      }
    });

    if class.super_name.is_some() {
      self.end_scope();
    }

    self.state.class = old_class_state;
  }

  fn resolve_expr(&mut self, expr: &Expr) {
    use Expr::*;
    match &expr {
      Lit(_) => {}
      Var(var) => {
        if self.query(&var.name, BindingState::Declared(var.span)) {
          self.error(
            ErrorType::Error,
            var.name.span,
            format!(
              "Cannot read local variable `{}` in its own initializer",
              var.name
            ),
          )
        };
        self.resolve_binding(&var.name);
      }
      Call(call) => {
        self.resolve_expr(&call.callee);
        let _ = call.args.iter().map(|arg| self.resolve_expr(&arg));
      },
      Get(get) => {
        self.resolve_expr(&get.obj);
      },
      Set(set) => {
        self.resolve_expr(&set.value);
        self.resolve_expr(&set.obj);
      },
      This(this) => {
        if self.state.class == ClassState::None {
          self.error(
            ErrorType::Error,
            this.span,
            "Illegal `this`: can't use `this` outside of a class"
          )
        }
        self.resolve_binding(&this.name);
      },
      Super(sup) => {
        match self.state.class {
          ClassState::None => self.error(
            ErrorType::Error,
              sup.super_ident.span,
              "Illegal `super`: can't use `super` outside of a class",
          ),
          ClassState::Class => self.error(
            ErrorType::Error,
              sup.super_ident.span,
              "Illegal `super`: can't use `super` within a class with no superclass",
          ),
          _ => {}
        }
        self.resolve_binding(&sup.super_ident);
      }
      Assignment(assign) => {
        self.resolve_expr(&assign.value);
        self.resolve_binding(&assign.name);
      },
      Binary(binary) => {
        self.resolve_expr(&binary.left);
        self.resolve_expr(&binary.right);
      },
      Logical(logical) => {
        self.resolve_expr(&logical.left);
        self.resolve_expr(&logical.right);
      },
      Unary(unary) => self.resolve_expr(&unary.operand),
      Group(group) => self.resolve_expr(&group.expr),
      Lambda(lambda) => {
        self.declare(&lambda.decl.name);
        self.define(&lambda.decl.name);

        self.resolve_fun(&lambda.decl, FunctionState::Function);
      }
      // _ => {}
    }
  }
}

impl<'i> Resolver<'i> {
  pub fn new(interpreter: &'i mut Interpreter) -> Self {
    Self {
      interpreter,
      state: ResolverState::default(),
      scopes: Vec::new(),
      errors: Vec::new(),
    }
  }

  fn declare(&mut self, ident: &LoxIdent) {
    if self.scopes.is_empty() {
      return;
    }
    let Some(scope) = self.scopes.last_mut() else {
      unreachable!();
    };

    match scope.entry(ident.name.clone()) {
      Entry::Vacant(entry) => {
        entry.insert(BindingState::Declared(ident.span));
      }
      Entry::Occupied(_) => {
        self.error(
          ErrorType::Error,
          ident.span,
          format!("Cannot shadow `{}` in the same scope", ident.name),
        );
      }
    };
  }

  fn define(&mut self, ident: &LoxIdent) {
    if self.scopes.is_empty() {
      return;
    }
    let Some(scope) = self.scopes.last_mut() else {
      unreachable!();
    };

    match scope.get_mut(&ident.name) {
      Some(binding) => *binding = BindingState::Initialized(ident.span),
      None => {
        self.error(
          ErrorType::Error,
          ident.span,
          format!("Binding `{}` is not defined", ident.name),
        );
      }
    };
  }

  fn access(&mut self, ident: &LoxIdent) {
    if self.scopes.is_empty() {
      return;
    }
    let Some(scope) = self.scopes.last_mut() else {
      unreachable!();
    };

    match scope.get_mut(&ident.name) {
      Some(binding) => *binding = BindingState::Accessed,
      None => {
        self.error(
          ErrorType::Error,
          ident.span,
          format!("Binding `{}` is not defined", ident.name),
        );
      }
    };
  }

  fn initialize(&mut self, ident: impl Into<String>) {
    self
      .scopes
      .last_mut()
      .unwrap()
      .insert(ident.into(), BindingState::Accessed);
  }

  fn query(&mut self, ident: &LoxIdent, expected: BindingState) -> bool {
    self.scopes.last().and_then(|scope| scope.get(&ident.name)) == Some(&expected)
  }

  fn resolve_binding(&mut self, ident: &LoxIdent) {
    let mut accessed = false;
    for (depth, scope) in self.scopes.iter_mut().rev().enumerate() {
      if scope.contains_key(&ident.name) {
        if depth == 0 { accessed = true; }
        self.interpreter.resolve_local(ident, depth);
      }
    }
    if accessed {
      self.access(ident);
    }
  }

  fn resolve_fun(&mut self, decl: &stmt::FunDecl, state: FunctionState) {
    let old_function_state = mem::replace(&mut self.state.function, state);

    self.scoped(|this| {
      for param in &decl.params {
        this.declare(param);
        this.define(param);
      }

      this.resolve_stmts(&decl.body);
    });

    self.state.function = old_function_state;
  }

  /// One should ideally use `scoped`. Callers of `begin_scope` must also call `end_scope`.
  #[inline]
  fn begin_scope(&mut self) {
    self.scopes.push(HashMap::new());
  }

  #[inline]
  fn end_scope(&mut self) {
    self.scopes.pop();
  }

  fn scoped<I>(&mut self, inner: I)
  where
    I: FnOnce(&mut Self),
  {
    self.begin_scope();
    let res = inner(self);
    self.check_unused();
    self.end_scope();
    res
  }

  /// Reports any unused local variables
  fn check_unused(&mut self) {
    use BindingState::*;
    if let Some(scope) = self.scopes.last() {
      for (key, state) in scope.iter() {
        match state {
          Declared(span) | Initialized (span) => {
            self.errors.push(ResolveError {
              kind: ErrorType::Warning,
              message: format!("Unused variable `{}`", key),
              span: *span,
            })
          }
          _ => continue
        }
      }
    }
  }

  fn error(&mut self, kind: ErrorType, span: Span, message: impl Into<String>) {
    let message = message.into();
    self.errors.push(ResolveError { span, message, kind });
  }
}

#[derive(Debug, Copy, Clone, Eq)]
enum BindingState {
  Declared(Span),
  Initialized(Span),
  Accessed,
}

impl PartialEq for BindingState {
  fn eq(&self, other: &Self) -> bool {
    use BindingState::*;
    match (self, other) {
      (Declared(_), Declared(_)) => true,
      (Initialized(_), Initialized(_)) => true,
      (Accessed, Accessed) => true,
      _ => false
    }
  }
}

#[derive(Debug, Default)]
struct ResolverState {
  function: FunctionState,
  class: ClassState,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum FunctionState {
    None,
    Init,   // Class init
    Method, // Class method
    Function,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum ClassState {
    None,
    Class,
    SubClass,
}

macro_rules! impl_default_for_state {
  ($($name:ident),+) => {
      $(
          impl Default for $name {
              fn default() -> Self {
                  Self::None
              }
          }
      )+
  }
}

impl_default_for_state!(FunctionState, ClassState);