use crate::vertex::{VertexData, VertexFormat};

pub(crate) struct Object3d {
    pub name: String,
    pub format: VertexFormat,
    pub vertex_buffer: Vec<VertexData>,
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
    
    pub(crate) fn add_vertex(&mut self, new_vertex: VertexData) -> Result<(), String> {
        if self.format == VertexFormat::Unknown {
            self.format = new_vertex.format;
        } else if self.format != new_vertex.format {
            return Err(String::from("Compilation error: Unexpected vertex format change"));
        }
        
        let mut index = None;
        for i in 0..self.vertex_buffer.len() {
            if self.vertex_buffer[i] == new_vertex {
                index = Some(i);
                break;
            }
        }
        
        if let Some(i) = index {
            self.index_buffer.push(i as u64);
        } else {
            self.index_buffer.push(self.vertex_buffer.len() as u64);
            self.vertex_buffer.push(new_vertex);
        }
        
        Ok(())
    }
}
