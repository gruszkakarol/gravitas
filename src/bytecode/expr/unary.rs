use crate::{
    bytecode::{BytecodeFrom, BytecodeGenerator, GenerationResult, Opcode, Patch, PATCH},
    parser::expr::{Operator, Unary},
};

impl BytecodeFrom<Unary> for BytecodeGenerator {
    fn generate(&mut self, unary: &Unary) -> GenerationResult {
        let Unary { expr, operator } = unary;
        self.generate(expr)?;
        let opcode = match *operator {
            Operator::Bang => Opcode::Not,
            Operator::Minus => Opcode::Negate,
            _ => unreachable!(),
        };
        self.emit_code(opcode);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use crate::{
        bytecode::{test::generate_bytecode, Opcode, Value},
        parser::expr::{
            binary::{Binary, Operator},
            Atom, Expr,
        },
    };

    use super::*;

    #[test]
    fn expr_unary() {
        let ast = Unary {
            expr: Box::new(Expr::Atom(Atom::Number(10.0))),
            operator: Operator::Minus,
        };
        let (chunk, bytecode) = generate_bytecode(ast);
        assert_eq!(bytecode, vec![Opcode::Constant(0), Opcode::Negate]);
        assert_eq!(chunk.read_constant(0), &Value::Number(10.0));

        let ast = Unary {
            expr: Box::new(Expr::Binary(Binary {
                left: Box::new(Expr::Atom(Atom::Number(10.0))),
                operator: Operator::Plus,
                right: Box::new(Expr::Atom(Atom::Number(10.0))),
            })),
            operator: Operator::Minus,
        };

        let (chunk, bytecode) = generate_bytecode(ast);
        assert_eq!(
            bytecode,
            vec![
                Opcode::Constant(0),
                Opcode::Constant(1),
                Opcode::Add,
                Opcode::Negate
            ]
        );
        assert_eq!(chunk.read_constant(0), &Value::Number(10.0));
        assert_eq!(chunk.read_constant(1), &Value::Number(10.0));

        let ast = Unary {
            expr: Box::new(Expr::Atom(Atom::Bool(true))),
            operator: Operator::Bang,
        };

        let (_, bytecode) = generate_bytecode(ast);
        assert_eq!(bytecode, vec![Opcode::True, Opcode::Not,]);

        let ast = Unary {
            expr: Box::new(Expr::Binary(Binary {
                left: Box::new(Expr::Atom(Atom::Number(20.0))),
                operator: Operator::Plus,
                right: Box::new(Expr::Atom(Atom::Number(10.0))),
            })),
            operator: Operator::Bang,
        };

        let (chunk, bytecode) = generate_bytecode(ast);
        assert_eq!(
            bytecode,
            vec![
                Opcode::Constant(0),
                Opcode::Constant(1),
                Opcode::Add,
                Opcode::Not
            ]
        );

        assert_eq!(chunk.read_constant(0), &Value::Number(20.0));
        assert_eq!(chunk.read_constant(1), &Value::Number(10.0));
    }
}
