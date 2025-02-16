use crate::nan_safe_float::Float;

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub(crate) enum VertexFormat {
    Unknown,
    VertexP,
    VertexPN,
    VertexPT,
    VertexPNT,
}

impl VertexFormat {
    pub(crate) fn from_indices(indices: &(u64, u64, u64)) -> Self {
        match indices {
            (0, _tc, _n) => { panic!("Vertex format must have position index"); },
            (_p, 0, 0) => VertexFormat::VertexP,
            (_p, _tc, 0) => VertexFormat::VertexPT,
            (_p, 0, _n) => VertexFormat::VertexPN,
            (_ps, _tc, _n) => VertexFormat::VertexPNT,
        }
    }
}

pub(crate) struct VertexDataIndex {
    format: VertexFormat,
    pos: u64,
    normal: u64,
    tex_coord: u64,
}

impl VertexDataIndex {
    pub(crate) fn from_indices(indices: &(u64, u64, u64)) -> Self {
        Self {
            format: VertexFormat::from_indices(indices),
            pos: indices.0,
            normal: indices.2,
            tex_coord: indices.1,
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub(crate) struct VertexData {
    pub(crate) format: VertexFormat,
    pos: (Float, Float, Float),
    normal: Option<(Float, Float, Float)>,
    tex_coord: Option<(Float, Float)>,
}

impl VertexData {
    pub(crate) fn vertex_p_from_floats(x: Float, y: Float, z:Float) -> Self {
        VertexData {
            format: VertexFormat::VertexP,
            pos: (x, y, z),
            normal: None,
            tex_coord: None,
        }
    }
    
    pub(crate) fn vertex_pn_from_floats(
        px: Float, py: Float, pz: Float,
        nx: Float, ny: Float, nz: Float
    ) -> Self {
        VertexData {
            format: VertexFormat::VertexPN,
            pos: (px, py, pz),
            normal: Some((nx, ny, nz)),
            tex_coord: None,
        }
    }
    
    pub(crate) fn vertex_pt_from_floats(
        px: Float, py: Float, pz: Float,
        tx: Float, ty: Float
    ) -> Self {
        VertexData {
            format: VertexFormat::VertexPT,
            pos: (px, py, pz),
            normal: None,
            tex_coord: Some((tx, ty))
        }
    }
    
    pub(crate) fn vertex_pnt_from_floats(
        px: Float, py: Float, pz: Float,
        nx: Float, ny: Float, nz: Float,
        tx: Float, ty: Float
    )-> Self {
        VertexData {
            format: VertexFormat::VertexPNT,
            pos: (px, py, pz),
            normal: Some((nx, ny, nz)),
            tex_coord: Some((tx, ty))
        }
    }
    
    pub(crate) fn compile(
        index: VertexDataIndex, 
        position_buffer: &Vec<(Float, Float, Float)>,
        normal_buffer: &Vec<(Float, Float, Float)>,
        tex_coord_buffer: &Vec<(Float, Float)>
    ) -> Result<Self, String> {
        match index.format {
            VertexFormat::Unknown => Err(String::from("Cannot compile vertex when format is unknown")),
            VertexFormat::VertexP => VertexData::compile_vertex_p(index, position_buffer),
            VertexFormat::VertexPN => VertexData::compile_vertex_pn(index, position_buffer, normal_buffer),
            VertexFormat::VertexPT => VertexData::compile_vertex_pt(index, position_buffer, tex_coord_buffer),
            VertexFormat::VertexPNT => VertexData::compile_vertex_pnt(index, position_buffer, normal_buffer, tex_coord_buffer),
        }
    } 
    
    fn compile_vertex_p(
        index: VertexDataIndex,
        position_buffer: &Vec<(Float, Float, Float)>
    ) -> Result<Self, String> {
        let position = position_buffer.get(index.pos as usize - 1);
        if let None = position {
            Err(String::from("Bad position index"))
        } else {
            Ok(Self {
                format: VertexFormat::VertexP,
                pos: *position.unwrap(),
                normal: None,
                tex_coord: None
            })
        }
    }
    
    fn compile_vertex_pn(
        index: VertexDataIndex,
        position_buffer: &Vec<(Float, Float, Float)>,
        normal_buffer: &Vec<(Float, Float, Float)>
    ) -> Result<Self, String> {
        let position = position_buffer.get(index.pos as usize - 1);
        if let None = position {
            return Err(String::from("Bad position index"));
        }
        
        let normal = normal_buffer.get(index.normal as usize - 1);
        if let None = normal {
            return Err(String::from("Bad normal index"));
        }
        
        Ok(
            VertexData {
                format: VertexFormat::VertexPN,
                pos: *position.unwrap(),
                normal: normal.copied(),
                tex_coord: None
            }
        )
    }
    
    fn compile_vertex_pt(
        index: VertexDataIndex,
        position_buffer: &Vec<(Float, Float, Float)>,
        tex_coord_buffer: &Vec<(Float, Float)>
    ) -> Result<Self, String> {
        let position = position_buffer.get(index.pos as usize - 1);
        if let None = position {
            return Err(String::from("Bad position index"));
        }
        
        let tex_coord = tex_coord_buffer.get(index.tex_coord as usize - 1);
        if let None = tex_coord {
            return Err(String::from("Bad texture coordinate index"));
        }
        
        Ok(
            VertexData {
                format: VertexFormat::VertexPT,
                pos: *position.unwrap(),
                normal: None,
                tex_coord: tex_coord.copied()
            }
        )
    }
    
    fn compile_vertex_pnt(
        index: VertexDataIndex,
        position_buffer: &Vec<(Float, Float, Float)>,
        normal_buffer: &Vec<(Float, Float, Float)>,
        tex_coord_buffer: &Vec<(Float, Float)>
    ) -> Result<Self, String> {
        let position = position_buffer.get(index.pos as usize - 1);
        if let None = position {
            return Err(String::from("Bad position index"));
        }
        
        let normal = normal_buffer.get(index.normal as usize - 1);
        if let None = normal {
            return Err(String::from("Bad normal index"));
        }
        
        let tex_coord = tex_coord_buffer.get(index.tex_coord as usize - 1);
        if let None = tex_coord {
            return Err(String::from("Bad texture coordinate index"));
        }
        
        Ok(
            VertexData {
                format: VertexFormat::VertexPNT,
                pos: *position.unwrap(),
                normal: normal.copied(),
                tex_coord: tex_coord.copied()
            }
        )
    }
}