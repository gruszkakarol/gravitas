use std::collections::HashMap;

use anyhow::{Context, Result};
use derive_more::Display;
use logos::Logos;
use ordered_float::NotNan;

use crate::hashmap;

#[derive(Logos, Debug, Display, Clone, Hash, Eq, PartialEq)]
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
    BangEquals,
    #[token("<")]
    Less,
    #[token("<=")]
    LessEquals,
    #[token(">")]
    Greater,
    #[token(">=")]
    GreaterEquals,
    #[token("==")]
    Compare,
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
    Number(NotNan<f64>),
    #[regex("\"[a-zA-Z]+\"", | lex | lex.slice().parse())]
    Text(String),
    #[regex("[a-zA-Z]+", | lex | lex.slice().parse())]
    Identifier(String),
    #[error]
    // We can also use this variant to define whitespace,
    // or any other matches we wish to skip.
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Error,
}

#[derive(Display)]
pub enum AffixKind {
    Infix,
    Prefix,
}

type RulesMap = HashMap<Token, usize>;
type InfixRules = RulesMap;
type PrefixRules = RulesMap;

lazy_static! {
    static ref PREFIX_RULES: PrefixRules = hashmap!(
         Token::Minus => 7,
         Token::OpenParenthesis => 0
    );
    static ref INFIX_RULES: InfixRules = hashmap!(
         Token::Plus => 5,
         Token::Minus => 5,
         Token::Star => 6,
         Token::Divide => 6,
         Token::CloseParenthesis => 0
    );
}

impl Token {
    pub fn bp(&self, kind: AffixKind) -> Result<usize> {
        match kind {
            AffixKind::Prefix => PREFIX_RULES.get(self),
            AffixKind::Infix => INFIX_RULES.get(self),
        }
        .copied()
        .with_context(|| format!("No rule specified for token {} as an {}", self, kind))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// Token is able to find a rule for corresponding kind of affix.
    #[test]
    fn finds_rule() {
        assert_eq!(
            Token::Minus.bp(AffixKind::Infix).expect("Rule not found"),
            5
        );

        assert_eq!(
            Token::Minus.bp(AffixKind::Prefix).expect("Rule not found"),
            7
        );
    }

    /// It throws an error if it doesn't find a corresponding rule.
    #[test]
    #[should_panic(expected = "Rule not found")]
    fn rule_not_found() {
        Token::Error.bp(AffixKind::Infix).expect("Rule not found");
    }
}
