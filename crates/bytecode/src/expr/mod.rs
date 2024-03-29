use parser::parse::expr::{Expr, ExprKind};

use crate::{chunk::Constant, state::ScopeType, BytecodeFrom, BytecodeGenerator, Opcode};

mod atom;
mod binary;
mod flow_control;
mod unary;

impl BytecodeFrom<Vec<Expr>> for BytecodeGenerator {
    fn generate(&mut self, data: Vec<Expr>) -> crate::BytecodeGenerationResult {
        for expr in data {
            self.generate(expr)?;
        }
        Ok(())
    }
}

impl BytecodeFrom<Expr> for BytecodeGenerator {
    fn generate(&mut self, expr: Expr) -> crate::BytecodeGenerationResult {
        match *expr.kind {
            ExprKind::Atom(atomic_value) => {
                self.generate(atomic_value)?;
            }
            ExprKind::Binary { lhs, op, rhs } => {
                self.generate(lhs)?;
                self.generate(rhs)?;
                let operator_code = op.kind.into();
                self.write_opcode(operator_code);
            }
            ExprKind::Unary { op, rhs } => {
                self.generate(rhs)?;
                let operator_code = op.kind.into();
                self.write_opcode(operator_code);
            }
            ExprKind::If {
                condition,
                body,
                else_expr,
            } => {
                self.generate(condition)?;

                let jif_patch = self.emit_patch(Opcode::Jif(0));
                self.generate(body)?;
                let jp_patch = self.emit_patch(Opcode::Jp(0));
                self.patch(&jif_patch);

                if let Some(else_expr) = else_expr {
                    self.generate(else_expr)?;
                }

                self.patch(&jp_patch);
            }
            ExprKind::While { condition, body } => {
                self.enter_scope(ScopeType::Block);
                let start = self.curr_index();
                self.generate(condition)?;

                let jif = self.emit_patch(Opcode::Jif(0));
                self.generate(body)?;

                let end = self.curr_index();
                self.write_opcode(Opcode::Jp(-(end as isize - start as isize)));
                self.patch(&jif);
                // TODO: implement breaking from while loops with a value
                self.write_opcode(Opcode::Null);
                self.leave_scope();
            }
            ExprKind::Block { stmts, return_expr } => {
                self.generate(stmts)?;

                if let Some(return_expr) = return_expr {
                    self.generate(return_expr)?;
                } else {
                    self.write_opcode(Opcode::Null);
                }

                self.write_opcode(Opcode::Block(self.state.declared()));
            }
            ExprKind::Break { return_expr } => {
                if let Some(return_expr) = return_expr {
                    self.generate(return_expr)?;
                } else {
                    self.write_opcode(Opcode::Null);
                }
                self.emit_patch(Opcode::Break(0));
            }
            ExprKind::Continue => {
                let ending_index = self.curr_index();
                let starting_index = self.state.current_scope().starting_index;
                self.write_opcode(Opcode::Jp(starting_index as isize - ending_index as isize));
            }
            ExprKind::Call { callee, args } => {
                self.generate(args)?;
                self.generate(callee)?;
                self.write_opcode(Opcode::Call);
            }
            ExprKind::Return { value } => {
                if let Some(value) = value {
                    self.generate(value)?;
                } else {
                    self.write_opcode(Opcode::Null);
                }
                self.write_opcode(Opcode::Return);
            }
            ExprKind::Array { values } => {}
            ExprKind::Index { target, position } => {}
            ExprKind::GetProperty {
                target,
                identifier,
                is_method_call,
            } => {
                self.generate(target)?;
                self.write_constant(Constant::String(identifier.kind.clone()));
                self.write_opcode(Opcode::GetProperty {
                    bind_method: is_method_call,
                });
            }
            ExprKind::SetProperty {
                target,
                identifier,
                value,
            } => {
                self.generate(target)?;
                self.write_constant(Constant::String(identifier.kind.clone()));
                self.generate(value)?;

                self.write_opcode(Opcode::SetProperty(1));
            }
            ExprKind::Assignment { target, value } => {
                // TODO: If no additional logical will be added to it then it can just as well become a simple binary expression
                self.generate(target)?;
                self.generate(value)?;
                self.write_opcode(Opcode::Asg);
            }
            ExprKind::Closure { params, body } => {}
            ExprKind::ObjectLiteral { properties } => {
                let amount = properties.len();
                for (key, value) in properties {
                    self.generate(value)?;
                    self.write_constant(Constant::String(key));
                }
                self.write_opcode(Opcode::CreateObject(amount));
            }
        };
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use parser::parse::expr::{atom::AtomicValue, ExprKind};

    use crate::{
        chunk::Constant,
        test::{assert_bytecode_and_constants, box_node, declare_var, expr},
        BytecodeGenerator, Opcode,
    };

    #[test]
    fn it_patches_opcodes() {
        let mut generator = BytecodeGenerator::new();
        let patch = generator.emit_patch(Opcode::Jif(0));
        assert_eq!(patch.index, 0);
        // Adding some random opcodes to the chunk
        generator.write_opcode(Opcode::Add);
        generator.write_opcode(Opcode::Get);
        // We added some codes but the patched opcode remain the same
        assert_eq!(
            generator.clone().code().chunk.opcodes[patch.index],
            Opcode::Jif(0)
        );
        generator.patch(&patch);
        // After the patch the opcode internal value should be changed to +2
        // because we added two new opcodes and the jump should jump by 2
        assert_eq!(
            generator.clone().code().chunk.opcodes[patch.index],
            Opcode::Jif(2)
        );
    }

    #[test]
    fn generates_block_bytecode() {
        // If no return_expr is specified then block return null by default
        // Block also drops variables declared inside
        assert_bytecode_and_constants(
            box_node(ExprKind::Block {
                return_expr: None,
                stmts: vec![declare_var(
                    "foo".to_owned(),
                    expr(AtomicValue::Number(0.0)),
                )],
            }),
            vec![Opcode::Constant(0), Opcode::Null, Opcode::Block(1)], // expected_bytecode,
            vec![Constant::Number(0.0)],
        );
        // Otherwise block returns the last expression
        assert_bytecode_and_constants(
            box_node(ExprKind::Block {
                return_expr: Some(expr(AtomicValue::Number(5.0))),
                stmts: vec![],
            }),
            vec![Opcode::Constant(0), Opcode::Block(0)], // expected_bytecode,
            vec![Constant::Number(5.0)],
        );
    }
}
