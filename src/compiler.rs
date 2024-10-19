use crate::statement::{Statement, StatementType, StatementDataType};
use crate::object3d::{Object3d};
use crate::vertex_format::VertexFormat;

struct Compiler {
    default_name: String
}

impl Compiler {
    fn compile(&self, statements: &Vec<Statement>) -> Result<Vec<Object3d>, String> {
        let mut results: Vec<Object3d> = Vec::new();
        let mut cur_obj: Object3d = Object3d::from(self.default_name.clone());
        let mut global_vb: Vec<(f64, f64, f64)> = Vec::new();
        
        for statement in statements {
            if statement.statement_type == StatementType::VERTEX {
                global_vb.push(statement.data.number_3d_as_tuple().expect("Expected conversion"));
            } else if statement.statement_type == StatementType::FACE {
                let index_tuples = statement.data.face_as_index_tuples().expect("Expected conversion");
                for i in 0..3 {
                    let indices = index_tuples[i];
                    let format = VertexFormat::from_indices(&indices);
                    let vertex = Self::construct_vertex(&indices, &global_vb);
                    
                    if cur_obj.format != VertexFormat::Unknown {
                        if cur_obj.format != format {
                            return Err(String::from("Vertex format changes during polygon construction"));
                        }
                    } else {
                        cur_obj.format = format;
                    }
                    
                }
            }
        }

        Ok(results)
    }
    
    fn construct_vertex(
        (pos, normal, texcoord): &(u64, u64, u64), 
        pos_buffer: &Vec<(f64, f64, f64)>
    ) -> Vec<f64> {
        let mut result = Vec::new();
        
        let pos = pos_buffer[*pos as usize - 1];
        result.push(pos.0);
        result.push(pos.1);
        result.push(pos.2);
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn compiler_generates_single_unnamed_object_with_single_vertex_p_polygon() {
        let file_name = "test.obj";
        let expected_object_list = vec!(
            Object3d {
                name: String::from(file_name),
                format: VertexFormat::VertexP,
                vertex_buffer: vec!(-1.0, 0.0, -1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0),
                index_buffer: vec!(0, 1, 2),
            }
        );
        
        let statements = vec!(
            Statement::from(StatementType::VERTEX, StatementDataType::Number3D(-1.0, 0.0, -1.0), 1, 0),
            Statement::from(StatementType::VERTEX, StatementDataType::Number3D( 0.0, 0.0,  1.0), 1, 0),
            Statement::from(StatementType::VERTEX, StatementDataType::Number3D( 1.0, 0.0,  1.0), 1, 0),
            Statement::from(StatementType::FACE, StatementDataType::FacePTN(1, 0, 0, 2, 0, 0, 3, 0, 0), 1, 0),
        );

        compile_generates_unnamed_objects(String::from(file_name), expected_object_list, statements);
    }
    
    #[test]
    fn compile_generates_single_unnamed_object_with_multiple_vertex_p_polygons() {
        let file_name = "test.obj";
        let expected_object_list = vec!(
            Object3d {
                name: String::from(file_name),
                format: VertexFormat::VertexP,
                vertex_buffer: vec!(-1.0, 0.0, -1.0, -1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0, 0.0, -1.0),
                index_buffer: vec!(0, 1, 2, 2, 3, 0),
            }
        );
        
        let statements = vec!(
            Statement::from(StatementType::VERTEX, StatementDataType::Number3D(-1.0, 0.0, -1.0), 1, 0),
            Statement::from(StatementType::VERTEX, StatementDataType::Number3D(-1.0, 0.0,  1.0), 1, 0),
            Statement::from(StatementType::VERTEX, StatementDataType::Number3D( 1.0, 0.0,  1.0), 1, 0),
            Statement::from(StatementType::VERTEX, StatementDataType::Number3D( 1.0, 0.0, -1.0), 1, 0),
            Statement::from(StatementType::FACE, StatementDataType::FacePTN(1, 0, 0, 2, 0, 0, 3, 0, 0), 1, 0),
            Statement::from(StatementType::FACE, StatementDataType::FacePTN(3, 0, 0, 4, 0, 0, 1, 0, 0), 1, 0),
        );

        compile_generates_unnamed_objects(String::from(file_name), expected_object_list, statements);
    }
    
    fn compile_generates_unnamed_objects(
        file_name: String, 
        expected_object_list: Vec<Object3d>, 
        statements: Vec<Statement>
    ) {
        let c = Compiler {
            default_name: String::from(file_name)
        };
        let actual_object_list = c.compile(&statements);

        assert_eq!(
            false,
            actual_object_list.is_err(),
            "Compile returns successful result when given valid data"
        );

        assert_object_lists_eq(expected_object_list, actual_object_list.unwrap());
    }

    fn assert_object_lists_eq(expected_object_list: Vec<Object3d>, actual_object_list: Vec<Object3d>) {
        assert_eq!(
            expected_object_list.len(),
            actual_object_list.len(),
            "Compile returns expected number of objects"
        );

        for i in 0..expected_object_list.len() {
            let expected_object = &expected_object_list[i];
            let actual_object = &actual_object_list[i];

            assert_eq!(
                expected_object.name,
                actual_object.name,
                "Compile returns object {i} with expected name"
            );

            assert_eq!(
                expected_object.format,
                actual_object.format,
                "Compile returns object {i} with expected vertex format"
            );

            assert_eq!(
                expected_object.vertex_buffer,
                actual_object.vertex_buffer,
                "Compile returns object {i} with expected vertex buffer"
            );

            assert_eq!(
                expected_object.index_buffer,
                actual_object.index_buffer,
                "Compile returns object {i} with expected index buffer"
            );
        }
    }
}