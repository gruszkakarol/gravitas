use common::ProgramText;
use parser::{
    parse::{
        expr::{atom::AtomicValue, Expr, ExprKind},
        stmt::{Stmt, StmtKind},
        AstRef,
    },
    utils::error::{ParseError, ParseErrorCause},
};
use std::collections::HashMap;
use vm::gravitas_std::NATIVE_FUNCTIONS;

pub type AnalyzerResult<E> = Result<(), E>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ScopeType {
    Function,
    Loop,
    Global,
}

type Variables = HashMap<ProgramText, bool>;

#[derive(Debug, Clone)]
struct Scope {
    scope_type: ScopeType,
    variables: HashMap<ProgramText, bool>,
}

impl Scope {
    fn new(scope_type: ScopeType) -> Self {
        Self {
            scope_type,
            variables: HashMap::new(),
        }
    }

    fn global(global_variables: Variables) -> Self {
        Self {
            scope_type: ScopeType::Global,
            variables: global_variables,
        }
    }

    fn is_global(&self) -> bool {
        self.scope_type == ScopeType::Global
    }

    fn is_function(&self) -> bool {
        self.scope_type == ScopeType::Function
    }

    fn is_loop(&self) -> bool {
        self.scope_type == ScopeType::Loop
    }
}

#[derive(Default)]
pub struct Analyzer {
    scopes: Vec<Scope>,
}

impl Analyzer {
    pub fn new() -> Self {
        let variables: HashMap<ProgramText, bool> = NATIVE_FUNCTIONS
            .keys()
            .cloned()
            .map(|fun| (fun.into(), true))
            .collect();

        let scopes = vec![Scope::global(variables)];

        Self {
            scopes,
            ..Default::default()
        }
    }

    fn declare_var(&mut self, name: &str, initialized: bool) {
        self.current_scope_mut()
            .variables
            .insert(name.to_owned(), initialized);
    }

    fn find_var(&self, name: &ProgramText) -> Option<&bool> {
        for scope in self.scopes.iter().rev() {
            if let Some(var) = scope.variables.get(name) {
                return Some(var);
            }
        }

        None
    }

    fn enter_scope(&mut self, scope_type: ScopeType) {
        self.scopes.push(Scope::new(scope_type));
    }

    fn leave_scope(&mut self) {
        self.scopes.pop();
    }

    fn current_scope(&self) -> &Scope {
        self.scopes.last().unwrap()
    }

    fn current_scope_mut(&mut self) -> &mut Scope {
        self.scopes.last_mut().unwrap()
    }

    fn visit_expr(&mut self, expr: &Expr) -> AnalyzerResult<ParseError> {
        use ExprKind::*;
        let span = expr.span.clone();

        // TODO: just making it work. It probably should differentiate between the start and end span.
        let err = move |cause: ParseErrorCause| {
            Err(ParseError {
                span_end: span.clone(),
                span_start: span.clone(),
                cause,
            })
        };

        match &*expr.kind {
            Atom(AtomicValue::Identifier { name, .. }) => match self.find_var(name) {
                Some(false) => {
                    return err(ParseErrorCause::UsedBeforeInitialization);
                }
                Some(true) => {}
                None => {
                    return err(ParseErrorCause::NotDefined);
                }
            },
            Binary { lhs, rhs, .. } => {
                self.visit_expr(lhs)?;
                self.visit_expr(rhs)?;
            }
            Block { stmts, return_expr } => {
                for stmt in stmts {
                    self.visit_stmt(stmt)?;
                }

                if let Some(expr) = return_expr {
                    self.visit_expr(expr)?;
                }
            }
            While { condition, body } => {
                self.visit_expr(condition)?;
                self.enter_scope(ScopeType::Loop);
                self.visit_expr(body)?;
                self.leave_scope();
            }
            Continue => {
                if !self.current_scope().is_loop() {
                    return err(ParseErrorCause::UsedOutsideLoop);
                }
            }
            Break { return_expr } => {
                if !self.current_scope().is_loop() {
                    return err(ParseErrorCause::UsedOutsideLoop);
                }

                if let Some(expr) = return_expr {
                    self.visit_expr(expr)?;
                }
            }
            Return { value } => {
                if !self.current_scope().is_function() {
                    return err(ParseErrorCause::ReturnUsedOutsideFunction);
                }
                if let Some(value) = value {
                    self.visit_expr(value)?;
                }
            }
            Call { callee, args } => {
                self.visit_expr(callee)?;
                for arg in args {
                    self.visit_expr(arg)?;
                }
            }
            Unary { op, rhs } => {
                self.visit_expr(rhs)?;
            }
            If {
                condition,
                body,
                else_expr,
            } => {
                self.visit_expr(condition)?;
                self.visit_expr(body)?;
                if let Some(else_expr) = else_expr {
                    self.visit_expr(else_expr)?;
                }
            }
            Array { values } => {
                for value in values {
                    self.visit_expr(value)?;
                }
            }
            Index { target, position } => {
                self.visit_expr(target)?;
                self.visit_expr(position)?;
            }
            GetProperty {
                target,
                is_method_call,
                identifier,
            } => {
                self.visit_expr(target)?;
            }
            SetProperty {
                target,
                value,
                identifier,
            } => {
                self.visit_expr(target)?;
                self.visit_expr(value)?;
            }
            ObjectLiteral { properties } => {
                for (name, value) in properties {
                    self.visit_expr(value)?;
                }
            }
            Assignment { target, value } => {
                self.visit_expr(target)?;
                self.visit_expr(value)?;
            }
            Closure { params, body } => {
                self.enter_scope(ScopeType::Function);
                self.visit_expr(body)?;
                self.leave_scope();
            }
            _ => {}
        }
        Ok(())
    }

    fn visit_stmt(&mut self, stmt: &Stmt) -> AnalyzerResult<ParseError> {
        use StmtKind::*;

        match &*stmt.kind {
            VariableDeclaration { name, expr } => {
                self.declare_var(name, false);
                self.visit_expr(expr)?;
                self.declare_var(name, true);
            }

            FunctionDeclaration { body, name, .. } => {
                self.declare_var(name, true);
                self.enter_scope(ScopeType::Function);
                self.visit_expr(body)?;
                self.leave_scope();
            }
            Expression { expr } => {
                self.visit_expr(expr)?;
            }
        }
        Ok(())
    }

    pub fn analyze(&mut self, ast: AstRef) -> AnalyzerResult<Vec<ParseError>> {
        let mut errors: Vec<ParseError> = Vec::new();

        for stmt in ast {
            if let Err(e) = self.visit_stmt(stmt) {
                errors.push(e);
            }
        }

        if !errors.is_empty() {
            Err(errors)
        } else {
            Ok(())
        }
    }
}

pub fn analyze(ast: AstRef) -> AnalyzerResult<Vec<ParseError>> {
    let mut analyzer = Analyzer::new();
    analyzer.analyze(&ast)?;
    Ok(())
}

#[cfg(test)]
mod test {

    use parser::parse;

    use super::*;

    fn assert_err(code: &str, cause: ParseErrorCause) {
        let ast = parse(code).unwrap();
        assert_eq!(analyze(&ast).unwrap_err()[0].cause, cause);
    }

    #[test]
    fn errors() {
        use ParseErrorCause::*;
        assert_err("super;", UsedOutsideClass);
        assert_err("this;", UsedOutsideClass);
        assert_err("continue;", UsedOutsideLoop);
        assert_err("break;", UsedOutsideLoop);
        assert_err("let x = x + 1;", UsedBeforeInitialization);
        assert_err("x + 2;", NotDefined);
        assert_err("class Foo: Foo {}", CantInheritFromItself);
        assert_err("class Foo: DoesntExist {}", SuperclassDoesntExist);

        // evaluates errors inside blocks
        assert_err("{ continue; };", UsedOutsideLoop);
        // evaluates errors inside methods
        assert_err("class Foo { fn method() { continue; } }", UsedOutsideLoop);
        // evaluates errors inside functions
        assert_err("fn foo() { continue; }", UsedOutsideLoop);
        assert_err("return;", ReturnUsedOutsideFunction);
    }
}
