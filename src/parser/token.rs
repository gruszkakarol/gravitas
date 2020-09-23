use derive_more::Display;
use enum_as_inner::EnumAsInner;
use logos::Logos;

#[derive(Logos, Debug, Display, Clone, PartialEq, EnumAsInner)]
pub enum Token {
    #[token("|")]
    Bar,
    #[token("(")]
    OpenParenthesis,
    #[token(")")]
    CloseParenthesis,
    #[token("{")]
    OpenBrace,
    #[token("}")]
    CloseBrace,
    #[token(",")]
    Coma,
    #[token(".")]
    Dot,
    #[token("-")]
    Minus,
    #[token("+")]
    Plus,
    #[token("*")]
    Star,
    #[token("/")]
    Divide,
    #[token("%")]
    Modulo,
    #[token(";")]
    Semicolon,
    #[token("!")]
    Bang,
    #[token("!=")]
    BangEqual,
    #[token("<")]
    Less,
    #[token("<=")]
    LessEqual,
    #[token(">")]
    Greater,
    #[token(">=")]
    GreaterEqual,
    #[token("==")]
    Equal,
    #[token("=")]
    Assign,
    #[token("//")]
    Comment,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("false")]
    False,
    #[token("true")]
    True,
    #[token("var")]
    Var,
    #[token("while")]
    While,
    #[token("for")]
    For,
    #[token("and")]
    And,
    #[token("or")]
    Or,
    #[token("break")]
    Break,
    #[token("continue")]
    Continue,
    #[token("fn")]
    Function,
    #[token("return")]
    Return,
    #[token("class")]
    Class,
    #[token("super")]
    Super,
    #[token("this")]
    This,
    #[token("null")]
    Null,
    #[token("print")]
    Print,
    #[token("=>")]
    Arrow,
    #[regex("-?[0-9]*\\.?[0-9]+", | lex | lex.slice().parse())]
    Number(f64),
    #[regex("\"[^\"]*\"", | lex | lex.slice().parse())]
    Text(String),
    #[regex("[a-zA-Z]+", | lex | lex.slice().parse())]
    Identifier(String),
    #[error]
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Error,
}

#[derive(Display)]
pub enum Affix {
    Infix,
    Prefix,
}

impl Token {
    /// Get binding power of the token
    /// e.g infix minus should have smaller binding power
    /// than infix star, so we can parse the expressions in correct order.
    /// ```
    ///         2 + 2 * 8
    ///     should be parsed into   
    /// Expr::Binary(2, + , Expr::Binary(2 * 8))
    /// ```
    pub fn bp(&self, affix: Affix) -> usize {
        match affix {
            Affix::Prefix => match self {
                Token::Minus => 7,
                Token::Bang => 7,
                // Token::OpenParenthesis => 0,
                _ => 0,
            },
            Affix::Infix => match self {
                Token::Assign => 1,
                Token::BangEqual => 4,
                Token::Equal => 4,
                Token::Greater => 4,
                Token::GreaterEqual => 4,
                Token::Less => 4,
                Token::LessEqual => 4,
                Token::Plus => 5,
                Token::Minus => 5,
                Token::Star => 6,
                Token::Divide => 6,
                // Token::CloseParenthesis => 0,
                // Token::Semicolon => 0,
                // _ => return error(),
                _ => 0,
            },
        }
    }

    /// Helper to determine whether token is associated with parsing the statements
    pub fn is_stmt(&self) -> bool {
        matches!(
            self,
            Token::Var | Token::Print | Token::Class | Token::Function
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// Token is able to find a rule for corresponding kind of affix.
    /// It defaults to 0
    #[test]
    fn finds_rule() {
        assert_eq!(Token::Minus.bp(Affix::Infix), 5);
        assert_eq!(Token::Minus.bp(Affix::Prefix), 7);
        assert_eq!(Token::Error.bp(Affix::Prefix), 0);
    }
}