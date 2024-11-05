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
        
        // TODO: performance bottleneck ... replace O(x) linear search with something better
        // hashing the VertexData and using a map might yield a Ologn(x) search
        // remember to preserve ordering!!! ... index buffer refs vertices by position in vb
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

#[cfg(test)]
mod tests {
    use crate::f;
    use crate::nan_safe_float::Float;
    use super::*;
    use crate::vertex::VertexFormat;

    #[test]
    fn add_vertex_sets_object_vertex_format_when_unknown() {
        let mut obj = Object3d::from(String::from("Test"));
        
        obj.add_vertex(VertexData::vertex_p_from_floats(f!(0.0), f!(0.0), f!(0.0)))
            .expect("No error with valid data set");
        
        assert_eq!(
            obj.format,
            VertexFormat::VertexP,
            "add_vertex sets object vertex format from data set"
        );
    }
    
    #[test]
    fn add_vertex_returns_err_when_vertex_format_changes() {
        let mut obj = Object3d::from(String::from("Test"));
        
        obj.add_vertex(VertexData::vertex_p_from_floats(f!(0.0), f!(0.0), f!(0.0)))
            .expect("No error with valid data set");
        
        let result = obj.add_vertex(VertexData::vertex_pt_from_floats(f!(0.0), f!(0.0), f!(0.0), f!(0.0), f!(0.0)));
        assert!(
            result.is_err(),
            "add-vertex returns err when vertex format changes"
        )
    }
    
    #[test]
    fn add_vertex_adds_new_vertex_to_vertex_buffer_and_index_buffer() {
        let mut obj = Object3d::from(String::from("Test"));
        
        obj.add_vertex(VertexData::vertex_p_from_floats(f!(1.0), f!(1.0), f!(1.0)))
            .expect("No error with valid data set");
        
        assert_eq!(
            vec!(VertexData::vertex_p_from_floats(f!(1.0), f!(1.0), f!(1.0))),
            obj.vertex_buffer,
            "add vertex adds new vertex to vertex buffer"
        );
        assert_eq!(
            vec!(0u64),
            obj.index_buffer,
            "add vertex references new vertex via index buffer"
        );
    }
    
    #[test]
    fn add_vertex_references_duplicate_vertex_via_index_buffer() {
        let mut obj = Object3d::from(String::from("Test"));
        
        obj.add_vertex(VertexData::vertex_p_from_floats(f!(1.0), f!(1.0), f!(1.0)))
            .expect("No error with valid data set");
        
        obj.add_vertex(VertexData::vertex_p_from_floats(f!(1.0), f!(1.0), f!(1.0)))
            .expect("No error with valid data set");
        
        assert_eq!(
            vec!(VertexData::vertex_p_from_floats(f!(1.0), f!(1.0), f!(1.0))),
            obj.vertex_buffer,
            "add vertex adds new vertex to vertex buffer"
        );
        assert_eq!(
            vec!(0u64, 0u64),
            obj.index_buffer,
            "add vertex references duplicate vertex via index buffer"
        );
    }
}