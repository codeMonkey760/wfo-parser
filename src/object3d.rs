use crate::vertex::{VertexFormat};

pub(crate) struct Object3d {
    pub name: String,
    pub format: VertexFormat,
    pub vertex_buffer: Vec<f64>,
    pub index_buffer: Vec<u64>,
}

impl Object3d {
    pub(crate) fn from(name: String) -> Self {
        Self {
            name,
            format: VertexFormat::Unknown,
            vertex_buffer: Vec::new(),
            index_buffer: Vec::new(),
        }
    }
}
