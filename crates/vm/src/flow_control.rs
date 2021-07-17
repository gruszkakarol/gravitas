use common::Address;

use crate::{
    runtime_error::RuntimeErrorCause, runtime_value::RuntimeValue, MachineResult, OperationResult,
    VM,
};

impl VM {
    fn expect_address(&mut self, value: RuntimeValue) -> MachineResult<Address> {
        match value {
            RuntimeValue::Number(distance) => Ok(distance),
            _ => self.error(RuntimeErrorCause::ExpectedAddressValue),
        }
    }

    pub(crate) fn op_jif(&mut self) -> OperationResult {
        let (jump_value, condition) = self.pop_two_operands()?;
        let distance = self.expect_address(jump_value)?;

        if condition.eq(RuntimeValue::Bool(false), self)? {
            self.move_pointer(distance as isize)?;
        }

        Ok(())
    }

    pub(crate) fn op_jf(&mut self) -> OperationResult {
        let jump_value = self.pop_operand()?;
        let distance = self.expect_address(jump_value)?;
        assert!(distance.is_sign_positive());
        self.move_pointer(distance as isize)?;

        Ok(())
    }

    // pub(crate) fn op_jb(&mut self) -> OperationResult {}
}

#[cfg(test)]
mod test {
    use bytecode::{
        chunk::{Chunk, Constant},
        Opcode,
    };

    use crate::{runtime_value::RuntimeValue, test::new_vm, OperationResult};

    #[test]
    fn op_jif() -> OperationResult {
        let code = Chunk::new(
            vec![
                Opcode::Constant(0),
                Opcode::Constant(1),
                Opcode::Constant(2),
                Opcode::Jif,
            ],
            vec![
                Constant::Number(127.0),
                Constant::Bool(false),
                Constant::Number(3.0),
            ],
        );
        let mut vm = new_vm(code);
        assert_eq!(vm.ip, 0);
        assert!(vm.run()?.eq(RuntimeValue::Number(127.0), &mut vm)?);
        // opcodes advance the pointer to 0, 1, 2, and 3 and then we have a jump that advances by another 3 so from 3 to 6
        assert_eq!(vm.ip, 6);
        Ok(())
    }

    #[test]
    fn op_jf() -> OperationResult {
        let code = Chunk::new(
            vec![Opcode::Constant(0), Opcode::Constant(1), Opcode::Jf],
            vec![Constant::Number(127.0), Constant::Number(10.0)],
        );

        let mut vm = new_vm(code);
        assert_eq!(vm.ip, 0);
        assert!(vm.run()?.eq(RuntimeValue::Number(127.0), &mut vm)?);
        assert_eq!(vm.ip, 12);

        Ok(())
    }

    fn op_jb() {}
}