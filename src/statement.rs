use std::fmt;
use crate::nan_safe_float::Float;
use crate::vertex::VertexDataIndex;

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
    Number3D(Float, Float, Float),
    Number2D(Float, Float),
    Number(Float),
    FacePTN(u64, u64, u64, u64, u64, u64, u64, u64, u64),
    None(),
}

impl StatementDataType {
    pub(crate) fn number_3d_as_tuple(&self) -> Option<(Float, Float, Float)> {
        if let StatementDataType::Number3D(x, y, z) = self {
            return Some((*x, *y, *z));
        }
        
        None
    }
    
    pub(crate) fn number_2d_as_tuple(&self) -> Option<(Float, Float)> {
        if let StatementDataType::Number2D(x, y) = self {
            return Some((*x, *y));
        }
        
        None
    }
    
    pub(crate) fn face_as_index_tuples(&self) -> Option<Vec<VertexDataIndex>> {
        if let StatementDataType::FacePTN(xp, xn, xt, yp, yn, yt, zp, zn, zt) = self {
            let mut ret = Vec::new();
            ret.push(VertexDataIndex::from_indices(&(*xp, *xn, *xt)));
            ret.push(VertexDataIndex::from_indices(&(*yp, *yn, *yt)));
            ret.push(VertexDataIndex::from_indices(&(*zp, *zn, *zt)));
            
            return Some(ret);
        }
        
        None
    }
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