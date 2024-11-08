use std::{collections::VecDeque, mem};

use crate::tokenizer::{Operator, Token, TokenKind};

use super::{Expression, LetDeclKind};
pub struct Parser {
    tokens: std::collections::VecDeque<Token>,
}

#[derive(Debug, Clone)]
pub enum ParsingError {
    InQueueParsing,
    EndedTokens,
    UnexpectedToken(Token), //got a token and dont know how to handle it
    WrongToken {
        expected: TokenKind,
        received: TokenKind,
        token: Token,
    }, //got a token that shouldnt be here, such as let 5 = 5;
}

impl Parser {
    pub fn new() -> Self {
        Self {
            tokens: std::collections::VecDeque::new(),
        }
    }
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(0)
    }
    fn eat(&mut self) -> Result<Token, ParsingError> {
        if let Some(t) = self.tokens.pop_front() {
            Ok(t)
        } else {
            Err(ParsingError::EndedTokens)
        }
    }
    fn expect_exact(&mut self, tk: TokenKind) -> Result<Token, ParsingError> {
        let token = self.eat()?;
        if matches!(token.kind, ref tk) {
            Ok(token)
        } else {
            Err(ParsingError::WrongToken {
                expected: tk,
                received: token.kind.clone(),
                token,
            })
        }
    }
    fn expect(&mut self, tk: TokenKind) -> Result<Token, ParsingError> {
        let token = self.eat()?;
        if std::mem::discriminant(&token.kind) == std::mem::discriminant(&tk) {
            Ok(token)
        } else {
            Err(ParsingError::WrongToken {
                expected: tk,
                received: token.kind.clone(),
                token,
            })
        }
    }
    pub fn parse_tokens(
        &mut self,
        tokens: &mut VecDeque<Token>,
    ) -> Result<Expression, ParsingError> {
        if self.tokens.len() > 0 {
            return Err(ParsingError::InQueueParsing);
        }
        self.tokens.append(tokens);
        let mut expressions = Vec::new();
        while let Some(Token { kind, .. }) = self.peek() {
            match kind {
                TokenKind::Eof => break,
                _ => expressions.push(self.parse()?),
            }
            self.expect(TokenKind::SemiColon)?;
        }
        Ok(Expression::Program(expressions))
    }
    fn parse(&mut self) -> Result<Expression, ParsingError> {
        let tk = self.eat()?;
        match tk.kind {
            TokenKind::Let => self.parse_let_expr(),
            TokenKind::IntLit(_) | TokenKind::FloatLit(_) | TokenKind::Identifier(_) => {
                self.parse_primary(tk)
            }
            _ => Err(ParsingError::UnexpectedToken(tk)),
        }
    }
    fn parse_let_expr(&mut self) -> Result<Expression, ParsingError> {
        let varname = self.expect(TokenKind::Identifier(format!("")))?;
        let TokenKind::Identifier(identifier) = varname.kind else {
            unreachable!();
        };
        self.expect_exact(TokenKind::Operator(Operator::Eq))?;
        Ok(Expression::LetDecl {
            kind: LetDeclKind::Normal,
            varname: identifier,
            expr: Box::new(self.parse_expr()?),
        })
    }
    fn parse_expr(&mut self) -> Result<Expression, ParsingError> {
        let tk = self.eat()?;
        self.parse_primary(tk)
    }
    fn parse_primary(&self, token: Token) -> Result<Expression, ParsingError> {
        match token.kind {
            TokenKind::Identifier(vname) => Ok(Expression::Identifier(vname)),
            TokenKind::IntLit(lit) => Ok(Expression::IntLit(lit)),
            TokenKind::FloatLit(f) => Ok(Expression::FloatLit(f)),
            _ => Err(ParsingError::UnexpectedToken(token)),
        }
    }
}
