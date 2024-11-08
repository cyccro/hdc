use std::collections::VecDeque;

use crate::tokenizer::{Operator, Token, TokenKind};

use super::{Expression, LetDeclKind};
#[derive(Debug, Clone)]
pub struct ParseStep {
    line: usize,
    column: usize,
    fname: String,
    token: Token,
}
pub struct Parser {
    tokens: std::collections::VecDeque<Token>,
    pub backtrace: std::collections::VecDeque<ParseStep>,
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
    ExpectedBlock(Box<Expression>),
}

impl Parser {
    pub fn new() -> Self {
        Self {
            tokens: std::collections::VecDeque::new(),
            backtrace: std::collections::VecDeque::new(),
        }
    }
    fn create_step<T>(&mut self, line: u32, column: u32, token: Token, fname: T)
    where
        T: ToString,
    {
        self.backtrace.push_back(ParseStep {
            line: line as usize,
            column: column as usize,
            token,
            fname: fname.to_string(),
        })
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
        let Some(token) = self.tokens.pop_front() else {
            return Err(ParsingError::EndedTokens);
        };
        self.create_step(line!(), column!(), token.clone(), "expect_exact");
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
        let Some(token) = self.tokens.pop_front() else {
            return Err(ParsingError::EndedTokens);
        };
        self.create_step(line!(), column!(), token.clone(), "expect");
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
                _ => {
                    let expr = self.parse()?;
                    if let Expression::FuncDecl { block, .. } = &expr {
                        if let Expression::Block(_) = **block {
                            expressions.push(expr);
                            continue;
                        } else {
                            expressions.push(expr);
                            self.expect(TokenKind::SemiColon)?;
                        }
                    } else {
                        expressions.push(expr);
                        self.expect(TokenKind::SemiColon)?;
                    }
                }
            }
            self.backtrace.clear();
        }
        Ok(Expression::Program(expressions))
    }
    fn parse(&mut self) -> Result<Expression, ParsingError> {
        let tk = self.eat()?;
        self.create_step(line!(), column!(), tk.clone(), "parse");
        match tk.kind {
            TokenKind::Let => self.parse_let_expr(),
            TokenKind::Func => self.parse_func(tk),
            TokenKind::IntLit(_) | TokenKind::FloatLit(_) | TokenKind::Identifier(_) => {
                self.parse_secondary(tk)
            }
            _ => self.parse_primary(tk),
        }
    }
    fn parse_func(&mut self, tk: Token) -> Result<Expression, ParsingError> {
        self.create_step(line!(), column!(), tk, "parse_func");
        let TokenKind::Identifier(fname) = self.expect(TokenKind::Identifier(format!("")))?.kind
        else {
            unreachable!();
        };
        self.expect(TokenKind::OpenParen)?;
        let mut params = Vec::new();
        loop {
            if let Some(TokenKind::CloseParen) = self.peek().map(|t| &t.kind) {
                self.eat()?;
                break;
            }
            let TokenKind::Identifier(name) = self.expect(TokenKind::Identifier(format!("")))?.kind
            else {
                unreachable!();
            };
            self.expect(TokenKind::Colon)?;
            let TokenKind::Identifier(ptype) =
                self.expect(TokenKind::Identifier(format!("")))?.kind
            else {
                unreachable!();
            };
            params.push(super::Param { name, kind: ptype });
            if let Some(TokenKind::CloseParen) = self.peek().map(|t| &t.kind) {
                self.eat()?;
                break;
            } else {
                self.expect(TokenKind::SemiColon)?;
            }
        }
        let current = self.peek();
        let current_kind = current.map(|t| &t.kind);
        let mut expect_block = true;
        let rtype = match current_kind {
            Some(TokenKind::Colon) => {
                self.eat()?;
                let TokenKind::Identifier(rtype) =
                    self.expect(TokenKind::Identifier(format!("")))?.kind
                else {
                    unreachable!();
                };
                if matches!(
                    self.peek().map(|t| &t.kind),
                    Some(TokenKind::Operator(Operator::Eq))
                ) {
                    expect_block = false;
                    self.eat()?;
                }
                Some(rtype)
            }
            Some(TokenKind::Operator(Operator::Eq)) => {
                expect_block = false;
                self.eat()?;
                None
            }
            Some(TokenKind::OpenBrace) => None,
            None => return Err(ParsingError::EndedTokens),
            Some(_) => return Err(ParsingError::UnexpectedToken(current.unwrap().clone())),
        };
        let block = self.parse()?;
        if expect_block {
            if let Expression::Block(_) = block {
                Ok(Expression::FuncDecl {
                    identifier: fname,
                    params,
                    rtype,
                    block: Box::new(block),
                })
            } else {
                Err(ParsingError::ExpectedBlock(Box::new(block)))
            }
        } else {
            Ok(Expression::FuncDecl {
                identifier: fname,
                params,
                rtype,
                block: Box::new(block),
            })
        }
    }
    fn parse_secondary(&mut self, tk: Token) -> Result<Expression, ParsingError> {
        self.create_step(line!(), column!(), tk.clone(), "parse_secondary");
        self.parse_additive(tk)
    }
    fn parse_block(&mut self) -> Result<Expression, ParsingError> {
        self.create_step(
            line!(),
            column!(),
            self.peek().unwrap().clone(),
            "parse_block",
        );
        let mut exprs = Vec::new();
        loop {
            exprs.push(self.parse()?);
            let err = self.expect(TokenKind::SemiColon);
            if let Err(ParsingError::WrongToken { ref token, .. }) = err {
                if token.kind == TokenKind::CloseBrace {
                    break;
                } else {
                    return Err(err.unwrap_err());
                }
            }
            if let Some(TokenKind::CloseBrace) = self.peek().map(|t| &t.kind) {
                self.eat()?;
                break;
            }
        }
        Ok(Expression::Block(exprs))
    }
    fn parse_additive(&mut self, tk: Token) -> Result<Expression, ParsingError> {
        self.create_step(line!(), column!(), tk.clone(), "parse_additive");
        let mut left = self.parse_multiplicative(tk)?;
        loop {
            let Some(current) = self.peek() else {
                break;
            };
            if let TokenKind::Operator(operator @ (Operator::Plus | Operator::Minus)) = current.kind
            {
                self.eat()?;
                left = Expression::BinExpr {
                    lhs: Box::new(left),
                    rhs: Box::new({
                        let tk = self.eat()?;
                        self.parse_multiplicative(tk)?
                    }),
                    op: operator,
                }
            } else {
                break;
            }
        }
        Ok(left)
    }
    fn parse_multiplicative(&mut self, tk: Token) -> Result<Expression, ParsingError> {
        self.create_step(line!(), column!(), tk.clone(), "parse_multiplicative");
        let mut left = self.parse_primary(tk)?;
        loop {
            let Some(current) = self.peek() else {
                break;
            };
            if let TokenKind::Operator(operator @ (Operator::Star | Operator::Bar)) = current.kind {
                self.eat()?;
                left = Expression::BinExpr {
                    lhs: Box::new(left),
                    rhs: Box::new({
                        let tk = self.eat()?;
                        self.parse_primary(tk)?
                    }),
                    op: operator,
                }
            } else {
                break;
            }
        }
        Ok(left)
    }
    fn parse_let_expr(&mut self) -> Result<Expression, ParsingError> {
        self.create_step(
            line!(),
            column!(),
            self.peek().unwrap().clone(),
            "parse_let_expr",
        );
        let TokenKind::Identifier(varname) = self.expect(TokenKind::Identifier(format!("")))?.kind
        else {
            unreachable!()
        };
        self.expect_exact(TokenKind::Operator(Operator::Eq))?;
        Ok(Expression::LetDecl {
            kind: LetDeclKind::Normal,
            varname,
            expr: Box::new(self.parse()?),
        })
    }
    fn parse_primary(&mut self, token: Token) -> Result<Expression, ParsingError> {
        self.create_step(line!(), column!(), token.clone(), "parse_primary");
        match token.kind {
            TokenKind::Identifier(vname) => Ok(Expression::Identifier(vname)),
            TokenKind::IntLit(lit) => Ok(Expression::IntLit(lit)),
            TokenKind::FloatLit(f) => Ok(Expression::FloatLit(f)),
            TokenKind::Operator(Operator::Minus) => {
                Ok(Expression::Negative(Box::new(self.parse()?)))
            }
            TokenKind::OpenParen => {
                let r = Ok(self.parse()?);
                self.expect(TokenKind::CloseParen)?;
                r
            }
            TokenKind::OpenBrace => Ok(self.parse_block()?),
            _ => Err(ParsingError::UnexpectedToken(token)),
        }
    }
}

impl std::fmt::Display for ParseStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Parsing error: function = {}({:?}) at <{}:{}>",
            self.fname, self.token.kind, self.line, self.column
        )
    }
}
