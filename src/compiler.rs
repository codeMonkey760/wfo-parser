use crate::statement::{Statement, StatementType, StatementDataType};
use crate::object3d::{Object3d};
use crate::vertex::{VertexData, VertexFormat};
use crate::nan_safe_float::Float;

struct Compiler {
    default_name: String,
    cur_obj: Option<Object3d>,
    position_buffer: Vec<(Float, Float, Float)>,
    normal_buffer: Vec<(Float, Float, Float)>,
    tex_coord_buffer: Vec<(Float, Float)>,
}

impl Compiler {
    fn from_default_name(new_default_name: &String) -> Self {
        Compiler {
            default_name: new_default_name.clone(),
            cur_obj: None,
            position_buffer: Vec::new(),
            normal_buffer: Vec::new(),
            tex_coord_buffer: Vec::new(),
        }
    }
    
    fn compile(&mut self, statements: &Vec<Statement>) -> Result<Vec<Object3d>, String> {
        let mut results: Vec<Object3d> = Vec::new();
        
        for statement in statements {
            match statement.statement_type {
                StatementType::COMMENT => {/*comments don't have side effects ... so ignore?*/}
                StatementType::MTLLIB => {/*ignore these*/}
                StatementType::OBJECT => {self.handle_object_statement(statement, &mut results)?}
                StatementType::VERTEX => {self.handle_vertex_statement(statement)?}
                StatementType::NORMAL => {self.handle_normal_statement(statement)?}
                StatementType::TEXCOORD => {self.handle_tex_coord_statement(statement)?}
                StatementType::USEMTL => {/*TODO: implement material support*/}
                StatementType::FACE => {self.handle_face_statement(statement)?}
                StatementType::ILLUM => {/*ignore these*/}
            }
        }
        self.clean_up(&mut results)?;
        
        Ok(results)
    }
    
    fn handle_vertex_statement(&mut self, statement: &Statement) -> Result<(), String> {
        self.position_buffer.push(statement.data.number_3d_as_tuple().expect("Expected conversion"));
        
        Ok(())
    }
    
    fn handle_normal_statement(&mut self, statement: &Statement) -> Result<(), String> {
        self.normal_buffer.push(statement.data.number_3d_as_tuple().expect("Expected conversion"));
        
        Ok(())
    }
    
    fn handle_tex_coord_statement(&mut self, statement: &Statement) -> Result<(), String> {
        self.tex_coord_buffer.push(statement.data.number_2d_as_tuple().expect("Expected conversion"));
        
        Ok(())
    }
    
    fn handle_object_statement(&mut self, statement: &Statement, results: &mut Vec<Object3d>) -> Result<(), String> {
        let name = match &statement.data {
            StatementDataType::String(x) => x,
            _ => {return Err(String::from("Object statement did not have string name"))},
        };
        
        let current_obj = self.cur_obj.take();
        if let Some(x) = current_obj {
            results.push(x);
        }
        
        self.cur_obj = Some(Object3d::from(name.clone()));
        
        Ok(())
    }
    
    fn handle_face_statement(&mut self, statement: &Statement) -> Result<(), String> {
        let current_obj = self.cur_obj.get_or_insert(Object3d::from(self.default_name.clone()));
        let face_indices = statement.data.face_as_index_tuples().expect("Expected conversion");
        let pos_buffer = &self.position_buffer;
        let normal_buffer = &self.normal_buffer;
        let tex_coord_buffer = &self.tex_coord_buffer;
        
        for vertex_indices in face_indices {
            let vertex = VertexData::compile(vertex_indices, pos_buffer, &normal_buffer, &tex_coord_buffer).expect("Expected vertex compilation");
            
            let add_vertex_result = current_obj.add_vertex(vertex);
            if add_vertex_result.is_err() {
                return Err(add_vertex_result.err().unwrap());
            }
        }
        
        Ok(())
    }
    
    fn clean_up(&mut self, results: &mut Vec<Object3d>) -> Result<(), String> {
        let current_obj = self.cur_obj.take();
        
        if let Some(x) = current_obj {
            results.push(x);
        }
        
        Ok(())
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

        compile_generates_objects(String::from(file_name), expected_object_list, statements);
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

        compile_generates_objects(String::from(file_name), expected_object_list, statements);
    }
    
    #[test]
    fn compile_generates_single_named_object_with_vertex_p_polygons() {
        let object_name = String::from("Object1");
        let expected_object_list = vec!(
            Object3d {
                name: object_name.clone(),
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
            Statement::from(StatementType::OBJECT, StatementDataType::String(object_name), 1, 0),
            Statement::from(StatementType::FACE, StatementDataType::FacePTN(1, 0, 0, 2, 0, 0, 3, 0, 0), 1, 0),
        );

        compile_generates_objects(String::from("test.obj"), expected_object_list, statements);
    }
    
    #[test]
    fn compile_generates_multiple_named_objects_with_vertex_p_polygons() {
        let object_1_name = "object1";
        let object_2_name = "object2";
        
        let expected_object_list = vec!(
            Object3d {
                name: String::from(object_1_name),
                format: VertexFormat::VertexP,
                vertex_buffer: vec!(
                    VertexData::vertex_p_from_floats(f!(-1.0), f!(0.0), f!(-1.0)), 
                    VertexData::vertex_p_from_floats(f!(0.0), f!(0.0), f!(1.0)),
                    VertexData::vertex_p_from_floats(f!(1.0), f!(0.0), f!(1.0)),
                ),
                index_buffer: vec!(0, 1, 2),
            },
            Object3d {
                name: String::from(object_2_name),
                format: VertexFormat::VertexP,
                vertex_buffer: vec!(
                    VertexData::vertex_p_from_floats(f!(1.0), f!(0.0), f!(1.0)), 
                    VertexData::vertex_p_from_floats(f!(0.0), f!(0.0), f!(1.0)),
                    VertexData::vertex_p_from_floats(f!(-1.0), f!(0.0), f!(-1.0)),
                ),
                index_buffer: vec!(0, 1, 2),
            },
        );
        
        let statements = vec!(
            Statement::from(StatementType::VERTEX, StatementDataType::Number3D(f!(-1.0), f!(0.0), f!(-1.0)), 1, 0),
            Statement::from(StatementType::VERTEX, StatementDataType::Number3D(f!(0.0), f!(0.0),  f!(1.0)), 1, 0),
            Statement::from(StatementType::VERTEX, StatementDataType::Number3D(f!(1.0), f!(0.0),  f!(1.0)), 1, 0),
            Statement::from(StatementType::OBJECT, StatementDataType::String(String::from(object_1_name)), 1, 0),
            Statement::from(StatementType::FACE, StatementDataType::FacePTN(1, 0, 0, 2, 0, 0, 3, 0, 0), 1, 0),
            Statement::from(StatementType::OBJECT, StatementDataType::String(String::from(object_2_name)), 1, 0),
            Statement::from(StatementType::FACE, StatementDataType::FacePTN(3, 0, 0, 2, 0, 0, 1, 0, 0), 1, 0),
        );
        
        compile_generates_objects(String::from("test.obj"), expected_object_list, statements);
    }
    
    #[test]
    fn compile_generates_single_object_with_vertex_pn_polygons() {
        let file_name = "test.obj";
        let expected_object_list = vec!(
            Object3d {
                name: String::from(file_name),
                format: VertexFormat::VertexPN,
                vertex_buffer: vec!(
                    VertexData::vertex_pn_from_floats(f!(-1.0), f!(0.0), f!(-1.0), f!(0.0), f!(0.0), f!(1.0)),
                    VertexData::vertex_pn_from_floats(f!(0.0), f!(0.0), f!(1.0), f!(0.0), f!(0.0), f!(1.0)),
                    VertexData::vertex_pn_from_floats(f!(1.0), f!(0.0), f!(1.0), f!(0.0), f!(0.0), f!(1.0)),
                ),
                index_buffer: vec!(0, 1, 2),
            }
        );
        
        let statements = vec!(
            Statement::from(StatementType::VERTEX, StatementDataType::Number3D(f!(-1.0), f!(0.0), f!(-1.0)), 1, 0),
            Statement::from(StatementType::VERTEX, StatementDataType::Number3D(f!(0.0), f!(0.0),  f!(1.0)), 1, 0),
            Statement::from(StatementType::VERTEX, StatementDataType::Number3D(f!(1.0), f!(0.0),  f!(1.0)), 1, 0),
            Statement::from(StatementType::NORMAL, StatementDataType::Number3D(f!(0.0), f!(0.0), f!(1.0)), 1, 0),
            Statement::from(StatementType::FACE, StatementDataType::FacePTN(1, 0, 1, 2, 0, 1, 3, 0, 1), 1, 0),
        );

        compile_generates_objects(String::from(file_name), expected_object_list, statements);
    }
    
    #[test]
    fn compile_generates_single_object_with_pt_polygons() {
        let file_name = "test.obj";
        let expected_object_list = vec!(
            Object3d {
                name: String::from(file_name),
                format: VertexFormat::VertexPT,
                vertex_buffer: vec!(
                    VertexData::vertex_pt_from_floats(f!(-1.0), f!(0.0), f!(-1.0), f!(0.0), f!(0.0)),
                    VertexData::vertex_pt_from_floats(f!(0.0), f!(0.0), f!(1.0), f!(0.0), f!(1.0)),
                    VertexData::vertex_pt_from_floats(f!(1.0), f!(0.0), f!(1.0), f!(1.0), f!(0.0)),
                ),
                index_buffer: vec!(0, 1, 2),
            }
        );
        
        let statements = vec!(
            Statement::from(StatementType::VERTEX, StatementDataType::Number3D(f!(-1.0), f!(0.0), f!(-1.0)), 1, 0),
            Statement::from(StatementType::VERTEX, StatementDataType::Number3D(f!(0.0), f!(0.0),  f!(1.0)), 1, 0),
            Statement::from(StatementType::VERTEX, StatementDataType::Number3D(f!(1.0), f!(0.0),  f!(1.0)), 1, 0),
            Statement::from(StatementType::TEXCOORD, StatementDataType::Number2D(f!(0.0), f!(0.0)), 1, 0),
            Statement::from(StatementType::TEXCOORD, StatementDataType::Number2D(f!(0.0), f!(1.0)), 1, 0),
            Statement::from(StatementType::TEXCOORD, StatementDataType::Number2D(f!(1.0), f!(0.0)), 1, 0),
            Statement::from(StatementType::FACE, StatementDataType::FacePTN(1, 1, 0, 2, 2, 0, 3, 3, 0), 1, 0),
        );

        compile_generates_objects(String::from(file_name), expected_object_list, statements);
    }
    
    #[test]
    fn compile_generates_single_object_with_vertex_pnt_polygons() {
        let file_name = "test.obj";
        let expected_object_list = vec!(
            Object3d {
                name: String::from(file_name),
                format: VertexFormat::VertexPNT,
                vertex_buffer: vec!(
                    VertexData::vertex_pnt_from_floats(
                        f!(-1.0), f!(0.0), f!(-1.0), f!(0.0), f!(0.0), f!(1.0), f!(0.0), f!(0.0)
                    ),
                    VertexData::vertex_pnt_from_floats(
                        f!(0.0), f!(0.0), f!(1.0), f!(0.0), f!(0.0), f!(1.0), f!(0.0), f!(1.0)
                    ),
                    VertexData::vertex_pnt_from_floats(
                        f!(1.0), f!(0.0), f!(1.0), f!(0.0), f!(0.0), f!(1.0), f!(1.0), f!(0.0)
                    ),
                ),
                index_buffer: vec!(0, 1, 2),
            }
        );
        
        let statements = vec!(
            Statement::from(StatementType::VERTEX, StatementDataType::Number3D(f!(-1.0), f!(0.0), f!(-1.0)), 1, 0),
            Statement::from(StatementType::VERTEX, StatementDataType::Number3D(f!(0.0), f!(0.0),  f!(1.0)), 1, 0),
            Statement::from(StatementType::VERTEX, StatementDataType::Number3D(f!(1.0), f!(0.0),  f!(1.0)), 1, 0),
            Statement::from(StatementType::NORMAL, StatementDataType::Number3D(f!(0.0), f!(0.0), f!(1.0)), 1, 0),
            Statement::from(StatementType::TEXCOORD, StatementDataType::Number2D(f!(0.0), f!(0.0)), 1, 0),
            Statement::from(StatementType::TEXCOORD, StatementDataType::Number2D(f!(0.0), f!(1.0)), 1, 0),
            Statement::from(StatementType::TEXCOORD, StatementDataType::Number2D(f!(1.0), f!(0.0)), 1, 0),
            Statement::from(StatementType::FACE, StatementDataType::FacePTN(1, 1, 1, 2, 2, 1, 3, 3, 1), 1, 0),
        );

        compile_generates_objects(String::from(file_name), expected_object_list, statements);
    }
    
    #[test]
    fn compiles_generates_multiple_named_objects() {
        let file_name = String::from("test.obj");
        let object_1_name = String::from("Object1");
        let object_2_name = String::from("Object2");
        
        let expected_object_list = vec!(
            Object3d {
                name: object_1_name.clone(),
                format: VertexFormat::VertexP,
                vertex_buffer: vec!(
                    VertexData::vertex_p_from_floats(f!(-1.0), f!(-1.0), f!(0.0)),
                    VertexData::vertex_p_from_floats(f!(0.0), f!(1.0), f!(0.0)),
                    VertexData::vertex_p_from_floats(f!(1.0), f!(-1.0), f!(0.0)),
                ),
                index_buffer: vec!(0, 1, 2),
            },
            Object3d {
                name: object_2_name.clone(),
                format: VertexFormat::VertexP,
                vertex_buffer: vec!(
                    VertexData::vertex_p_from_floats(f!(1.0), f!(-1.0), f!(0.0)),
                    VertexData::vertex_p_from_floats(f!(0.0), f!(1.0), f!(0.0)),
                    VertexData::vertex_p_from_floats(f!(-1.0), f!(-1.0), f!(0.0)),
                ),
                index_buffer: vec!(0, 1, 2),
            }
        );
        
        let statements = vec!(
            Statement::from(StatementType::VERTEX, StatementDataType::Number3D(f!(-1.0), f!(-1.0), f!(0.0)), 1, 0),
            Statement::from(StatementType::VERTEX, StatementDataType::Number3D(f!(0.0), f!(1.0), f!(0.0)), 1, 0),
            Statement::from(StatementType::VERTEX, StatementDataType::Number3D(f!(1.0), f!(-1.0), f!(0.0)), 1, 0),
            Statement::from(StatementType::OBJECT, StatementDataType::String(object_1_name.clone()), 1, 0),
            Statement::from(StatementType::FACE, StatementDataType::FacePTN(1, 0, 0, 2, 0, 0, 3, 0, 0), 1, 0),
            Statement::from(StatementType::OBJECT, StatementDataType::String(object_2_name.clone()), 1, 0),
            Statement::from(StatementType::FACE, StatementDataType::FacePTN(3, 0, 0, 2, 0, 0, 1, 0, 0), 1, 0),
        );

        compile_generates_objects(String::from(file_name), expected_object_list, statements);
    }
    
    fn compile_generates_objects(
        file_name: String, 
        expected_object_list: Vec<Object3d>, 
        statements: Vec<Statement>
    ) {
        let mut c = Compiler::from_default_name(&file_name);
        
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