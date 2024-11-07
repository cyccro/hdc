use std::collections::VecDeque;

use super::{Cursor, Operator, Token, TokenKind};

#[derive(Debug)]
pub enum TokenizationErrorKind {
    FoundUnexpectedEof,
    UnexpectedChar(char),
    InvalidDigit(String),
}

#[derive(Debug)]
pub struct TokenizationError {
    line: usize,
    column: usize,
    kind: TokenizationErrorKind,
}
impl TokenizationError {
    pub fn unexpected_char(char: char, cursor: &Cursor) -> Self {
        Self::new(TokenizationErrorKind::UnexpectedChar(char), cursor)
    }
    pub fn invalid_digit(buf: String, cursor: &Cursor) -> Self {
        Self::new(TokenizationErrorKind::InvalidDigit(buf), cursor)
    }
    pub fn unexpected_eof(cursor: &Cursor) -> Self {
        Self::new(TokenizationErrorKind::FoundUnexpectedEof, cursor)
    }
    pub fn new(kind: TokenizationErrorKind, cursor: &Cursor) -> Self {
        Self {
            kind,
            line: cursor.line(),
            column: cursor.column(),
        }
    }
}
pub struct Tokenizer {
    content: String,
}

impl Tokenizer {
    pub fn get_char<'a>(
        cursor: &'a mut Cursor,
        chars: &'a Vec<char>,
        advance: bool,
    ) -> Result<&'a char, TokenizationError> {
        let tk = chars
            .get(cursor.index())
            .ok_or(TokenizationError::unexpected_eof(cursor));
        if advance {
            cursor.advance();
        }
        tk
    }
    pub fn check_for_reserved(buf: String, cursor: &Cursor) -> Token {
        match &*buf {
            "let" => Token::let_token(cursor),
            _ => return Token::identifier(buf, cursor),
        }
    }
    pub fn new(content: String) -> Self {
        Self { content }
    }
    pub fn gen(&self) -> Result<VecDeque<Token>, TokenizationError> {
        let mut vec = VecDeque::new();
        let chars: Vec<char> = self.content.chars().collect();
        let mut cursor = Cursor::new();
        while let Some(chr) = chars.get(cursor.index()) {
            vec.push_back(match chr {
                ';' => Token::new(TokenKind::SemiColon, &cursor),
                '=' => Token::new(TokenKind::Operator(Operator::Eq), &cursor),
                '\n' => {
                    cursor.advance_line();
                    continue;
                }
                _ => {
                    if chr.is_whitespace() {
                        cursor.advance();
                        continue;
                    } else if chr.is_ascii_digit() {
                        self.get_digit_lit(&mut cursor, &chars)?
                    } else if chr.is_alphabetic() {
                        self.get_identifier(&mut cursor, &chars)?
                    } else {
                        return Err(TokenizationError::unexpected_char(*chr, &cursor));
                    }
                }
            });
            cursor.advance();
        }
        Ok(vec)
    }
    pub fn get_identifier(
        &self,
        cursor: &mut Cursor,
        chars: &Vec<char>,
    ) -> Result<Token, TokenizationError> {
        let mut buf = String::new();
        loop {
            let chr = Self::get_char(cursor, chars, true)?;
            if chr.is_alphanumeric() {
                buf.push(*chr);
            } else {
                cursor.backward();
                break;
            }
        }
        Ok(Self::check_for_reserved(buf, cursor))
    }
    pub fn get_digit_lit(
        &self,
        cursor: &mut Cursor,
        chars: &Vec<char>,
    ) -> Result<Token, TokenizationError> {
        let mut buf = String::new();
        let mut hasdot = false;
        let mut rnormal = true;
        loop {
            let chr = Self::get_char(cursor, chars, true)?;
            if chr.is_ascii_digit() || *chr == '.' {
                if hasdot && rnormal {
                    rnormal = false;
                }
                if *chr == '.' {
                    hasdot = true;
                }
                buf.push(*chr);
            } else {
                cursor.backward();
                break;
            }
        }
        if !rnormal {
            return Err(TokenizationError::invalid_digit(buf, &cursor));
        }
        cursor.backward();
        if hasdot {
            Ok(Token::float_lit(buf, &cursor))
        } else {
            Ok(Token::int_lit(buf, &cursor))
        }
    }
}
