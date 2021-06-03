use crate::{
    common::{
        combine,
        error::{Forbidden, ParseErrorCause},
    },
    parse::{
        expr::Expr,
        stmt::{Stmt, StmtKind},
        Node, ParseResult, Parser, StmtResult, Symbol,
    },
    token::{
        constants::{CLOSE_PARENTHESIS, OPEN_BRACKET, OPEN_PARENTHESIS},
        Token,
    },
};

impl<'t> Parser<'t> {
    // fn foo(a, b, c) => a + b + c
    // fn foo(a, b, c) {
    //  return a + b + c;
    // }

    pub(crate) fn parse_fun_declaration(&mut self) -> StmtResult {
        let fn_keyword = self.expect(Token::Function)?.span();
        let (name, _) = self.expect_identifier()?;
        let params = self.parse_params()?;
        if self.peek() != OPEN_BRACKET {
            self.expect(Token::Arrow)?;
        }
        let body = self.parse_expression()?;
        let span = combine(&fn_keyword, &body.span);

        Ok(Stmt::boxed(
            StmtKind::FunctionDeclaration { name, params, body },
            span,
        ))
    }
}

#[cfg(test)]
mod test {
    use crate::parse::expr::atom::AtomicValue;
    use crate::parse::expr::{Expr, ExprKind};
    use crate::parse::pieces::{Param, Params};
    use crate::parse::stmt::{Stmt, StmtKind};
    use crate::parse::Symbol;
    use crate::token::constants::OPEN_PARENTHESIS;
    use crate::{
        common::{
            error::{Expect, Forbidden, ParseErrorCause},
            test::parser::symbol,
        },
        parse::Parser,
        token::Token,
    };

    #[test]
    fn parser_parses_function_declarations() {
        let mut parser = Parser::new("fn");
        assert_eq!(
            parser
                .parse_fun_declaration()
                .expect_err("Parser expects function's name"),
            ParseErrorCause::Expected(Expect::Identifier)
        );

        let mut parser = Parser::new("fn foo");
        assert_eq!(
            parser
                .parse_fun_declaration()
                .expect_err("Parser expects parameters"),
            ParseErrorCause::Expected(Expect::Token(OPEN_PARENTHESIS))
        );

        let mut parser = Parser::new("fn foo()");
        assert_eq!(
            parser
                .parse_fun_declaration()
                .expect_err("Parser doesn't see start of a block expression so it expects arrow token indicating immediate return"),
            ParseErrorCause::Expected(Expect::Token(Token::Arrow))
        );

        let mut parser = Parser::new("fn foo() =>");

        assert_eq!(
            parser
                .parse_fun_declaration()
                .expect_err("Parser expects expression"),
            ParseErrorCause::Expected(Expect::Expression)
        );

        let mut parser = Parser::new("fn foo() => 2");
        let declaration = parser.parse_fun_declaration().unwrap();
        assert_eq!(
            declaration,
            Stmt::boxed(
                StmtKind::FunctionDeclaration {
                    name: Symbol::default(),
                    params: Params::new(vec![], 6..8),
                    body: Expr::boxed(ExprKind::Atom(AtomicValue::Number(2.0)), 12..13)
                },
                0..13
            )
        );

        let mut parser = Parser::new("fn foo(a,b){ 2 }");
        let declaration = parser.parse_fun_declaration().unwrap();
        let fun_node = Stmt::boxed(
            StmtKind::FunctionDeclaration {
                name: symbol(0),
                params: Params::new(
                    vec![Param::new(symbol(1), 7..8), Param::new(symbol(2), 9..10)],
                    6..10,
                ),
                body: Expr::boxed(
                    ExprKind::Block {
                        stmts: vec![],
                        return_expr: Some(Expr::boxed(
                            ExprKind::Atom(AtomicValue::Number(2.0)),
                            13..14,
                        )),
                    },
                    11..16,
                ),
            },
            0..16,
        );

        assert_eq!(declaration, fun_node);

        let mut parser = Parser::new("fn foo(a,b){ 2 }");
        assert_eq!(
            parser.parse_stmt().expect(
                "Parser should parse function body whenever it encounters function keyword"
            ),
            fun_node
        )
    }
}
