use crate::statement::{Statement, StatementType, StatementDataType};
use crate::object3d::{Object3d};
use crate::vertex::{VertexData, VertexFormat};
use crate::nan_safe_float::Float;

struct Compiler {
    default_name: String
}

impl Compiler {
    fn compile(&self, statements: &Vec<Statement>) -> Result<Vec<Object3d>, String> {
        let mut results: Vec<Object3d> = Vec::new();
        let mut cur_obj: Object3d = Object3d::from(self.default_name.clone());
        let mut pos_buffer: Vec<(Float, Float, Float)> = Vec::new();
        let normal_buffer: Vec<(Float, Float, Float)> = Vec::new();
        let tex_coord_buffer: Vec<(Float, Float)> = Vec::new();
        
        for statement in statements {
            if statement.statement_type == StatementType::VERTEX {
                pos_buffer.push(statement.data.number_3d_as_tuple().expect("Expected conversion"));
            } else if statement.statement_type == StatementType::FACE {
                let face_indices = statement.data.face_as_index_tuples().expect("Expected conversion");
                for vertex_indices in face_indices {
                    let vertex = VertexData::compile(vertex_indices, &pos_buffer, &normal_buffer, &tex_coord_buffer).expect("Expected vertex compilation");
                    
                    let add_vertex_result = cur_obj.add_vertex(vertex);
                    if add_vertex_result.is_err() {
                        return Err(add_vertex_result.err().unwrap());
                    }
                }
            }
        }
        
        results.push(cur_obj);

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use crate::f;
    use super::*;
    
    #[test]
    fn compiler_generates_single_unnamed_object_with_single_vertex_p_polygon() {
        let file_name = "test.obj";
        let expected_object_list = vec!(
            Object3d {
                name: String::from(file_name),
                format: VertexFormat::VertexP,
                vertex_buffer: vec!(
                    VertexData::vertex_p_from_floats(f!(-1.0), f!(0.0), f!(-1.0)), 
                    VertexData::vertex_p_from_floats(f!(0.0), f!(0.0), f!(1.0)),
                    VertexData::vertex_p_from_floats(f!(1.0), f!(0.0), f!(1.0)),
                ),
                index_buffer: vec!(0, 1, 2),
            }
        );
        
        let statements = vec!(
            Statement::from(StatementType::VERTEX, StatementDataType::Number3D(f!(-1.0), f!(0.0), f!(-1.0)), 1, 0),
            Statement::from(StatementType::VERTEX, StatementDataType::Number3D(f!(0.0), f!(0.0),  f!(1.0)), 1, 0),
            Statement::from(StatementType::VERTEX, StatementDataType::Number3D(f!(1.0), f!(0.0),  f!(1.0)), 1, 0),
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
                vertex_buffer: vec!(
                    VertexData::vertex_p_from_floats(f!(-1.0), f!(0.0), f!(-1.0)),
                    VertexData::vertex_p_from_floats(f!(-1.0), f!(0.0), f!(1.0)),
                    VertexData::vertex_p_from_floats(f!(1.0), f!(0.0), f!(1.0)),
                    VertexData::vertex_p_from_floats(f!(1.0), f!(0.0), f!(-1.0)),
                ),
                index_buffer: vec!(0, 1, 2, 2, 3, 0),
            }
        );
        
        let statements = vec!(
            Statement::from(StatementType::VERTEX, StatementDataType::Number3D(f!(-1.0), f!(0.0), f!(-1.0)), 1, 0),
            Statement::from(StatementType::VERTEX, StatementDataType::Number3D(f!(-1.0), f!(0.0),  f!(1.0)), 1, 0),
            Statement::from(StatementType::VERTEX, StatementDataType::Number3D(f!(1.0), f!(0.0), f!(1.0)), 1, 0),
            Statement::from(StatementType::VERTEX, StatementDataType::Number3D(f!(1.0), f!(0.0), f!(-1.0)), 1, 0),
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