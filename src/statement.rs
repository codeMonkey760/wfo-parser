use std::fmt;

#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum StatementType {
    COMMENT,
    MTLLIB,
    OBJECT,
    VERTEX,
    NORMAL,
    TEXCOORD,
    USEMTL,
    FACE,
    ILLUM,
}

#[derive(PartialEq, Debug, Clone)]
pub(crate) enum StatementDataType {
    String(String),
    Number3D(f64, f64, f64),
    Number2D(f64, f64),
    Number(f64),
    FacePTN(u64, u64, u64, u64, u64, u64, u64, u64, u64),
    None(),
}

impl fmt::Display for StatementType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            StatementType::COMMENT => "comment",
            StatementType::MTLLIB => "mtllib",
            StatementType::OBJECT => "object",
            StatementType::VERTEX => "vertex",
            StatementType::NORMAL => "normal",
            StatementType::TEXCOORD => "texcoord",
            StatementType::USEMTL => "usemtl",
            StatementType::FACE => "face",
            StatementType::ILLUM => "illum",
        })
    }
}

pub(crate) struct Statement {
    pub(crate) statement_type: StatementType,
    pub(crate) data: StatementDataType,
    pub(crate) line_number: u64,
    pub(crate) line_position: u64,
}

impl Statement {
    pub(crate) fn from(
        statement_type: StatementType,
        data: StatementDataType,
        line_number: u64,
        line_position: u64,
    ) -> Statement {
        Statement {
            statement_type,
            data,
            line_number,
            line_position,
        }
    }
}