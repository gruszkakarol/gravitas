use parser::parse::stmt::{Stmt, StmtKind};

use crate::{BytecodeFrom, BytecodeGenerationResult, BytecodeGenerator};

impl BytecodeFrom<Stmt> for BytecodeGenerator {
    fn generate(&mut self, stmt: Stmt) -> BytecodeGenerationResult {
        match *stmt.kind {
            StmtKind::Expression { expr } => {
                self.generate(expr)?;
            }
            StmtKind::VariableDeclaration { name, expr } => {}
            StmtKind::FunctionDeclaration { name, params, body } => {}
            StmtKind::ClassDeclaration {
                name,
                super_class,
                methods,
            } => {}
        }
        Ok(())
    }
}
