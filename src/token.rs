use std::fmt;
use std::fmt::Formatter;
use crate::nan_safe_float::Float;

#[derive(Eq, PartialEq, Debug, Ord, PartialOrd, Clone, Copy)]
pub enum TokenType {
    COMMENT,
    MTLLIB,
    OBJECT,
    VERTEX,
    NORMAL,
    TEXCOORD,
    USEMTL,
    FACE,
    ILLUM,
    NUMBER,
    STRING,
    POLYGON,
    SEPARATOR,
    LINEBREAK,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TokenType::COMMENT => { f.write_str("COMMENT") },
            TokenType::MTLLIB => { f.write_str("MTLLIB") },
            TokenType::OBJECT => { f.write_str("OBJECT") },
            TokenType::VERTEX => { f.write_str("VERTEX") },
            TokenType::NORMAL => { f.write_str("NORMAL") },
            TokenType::TEXCOORD => { f.write_str("TEXCOORD") },
            TokenType::USEMTL => { f.write_str("USEMTL") },
            TokenType::FACE => { f.write_str("FACE") },
            TokenType::ILLUM => { f.write_str("ILLUM") },
            TokenType::NUMBER => { f.write_str("NUMBER") },
            TokenType::STRING => { f.write_str("STRING") },
            TokenType::POLYGON => { f.write_str("POLYGON") },
            TokenType::SEPARATOR => { f.write_str("SEPARATOR") },
            TokenType::LINEBREAK => { f.write_str("LINEBREAK") },
        }
    }
}

impl TokenType {
    pub(crate) fn from_str(name: &str) -> Option<Self> {
        match name {
            "comment" => Some(TokenType::COMMENT),
            "mtllib" => Some(TokenType::MTLLIB),
            "o" => Some(TokenType::OBJECT),
            "v" => Some(TokenType::VERTEX),
            "vn" => Some(TokenType::NORMAL),
            "vt" => Some(TokenType::TEXCOORD),
            "usemtl" => Some(TokenType::USEMTL),
            "f" => Some(TokenType::FACE),
            "s" => Some(TokenType::ILLUM),
            _ => None
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub(crate) enum TokenDataType {
    String(String),
    Number(Float),
    VertexPTN(u64, u64, u64),
    None()
}

#[derive(PartialEq)]
pub(crate) struct Token {
    pub(crate) token_type: TokenType,
    pub(crate) data: TokenDataType,
    pub(crate) line_number: u64,
    pub(crate) line_position: u64,
}

impl Token {
    pub(crate) fn from(
        token_type: TokenType,
        data: TokenDataType,
        line_number: u64,
        line_position: u64
    ) -> Token {
        Token {
            token_type,
            data,
            line_number,
            line_position,
        }
    }
}
