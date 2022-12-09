use parser::parse::expr::{Expr, ExprKind};

use crate::{BytecodeFrom, BytecodeGenerator};

impl BytecodeFrom<Expr> for BytecodeGenerator {
    fn generate(&mut self, expr: Expr) -> crate::BytecodeGenerationResult {
        match *expr.kind {
            ExprKind::Atom(atomic_value) => {}
            ExprKind::Binary { lhs, op, rhs } => {}
            ExprKind::Unary { op, rhs } => {}
            ExprKind::Block { stmts, return_expr } => {}
            ExprKind::If {
                condition,
                body,
                else_expr,
            } => {}
            ExprKind::While { condition, body } => {}
            ExprKind::Break { return_expr } => {}
            ExprKind::Continue => {}
            ExprKind::Call { callee, args } => {}
            ExprKind::Return { value } => {}
            ExprKind::Array { values } => {}
            ExprKind::Index { target, position } => {}
            ExprKind::Property { target, paths } => {}
            ExprKind::Assignment { target, value } => {}
            ExprKind::Closure { params, body } => {}
            ExprKind::Super => {}
            ExprKind::This => {}
        };
        Ok(())
    }
}
