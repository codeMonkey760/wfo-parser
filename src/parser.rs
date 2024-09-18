use crate::token::{Token, TokenType, TokenDataType};
use crate::statement::{Statement, StatementDataType, StatementType};

struct Parser {
    statement_type: Option<StatementType>,
    statement_data: StatementDataType,
    statement_line_number: u64,
    statement_line_position: u64,
    data_buffer: Vec<f64>,
    index_buffer: Vec<u64>,
    parsed_token_count: u64,
    next_expected_token: TokenType,
}

impl Default for Parser {
    fn default() -> Self {
        Parser {
            statement_type: None,
            statement_data: StatementDataType::None(),
            statement_line_number: 0,
            statement_line_position: 0,
            data_buffer: Vec::new(),
            index_buffer: Vec::new(),
            parsed_token_count: 0,
            next_expected_token: TokenType::COMMENT,
        }
    }
}

impl Parser {
    fn new() -> Self {
        Default::default()
    }

    pub fn parse_tokens(
        mut self,
        tokens: &Vec<Token>,
    ) -> Result<Vec<Statement>, String> {
        let mut parsed_statements = Vec::new();

        for cur_token in tokens {
            let parse_result = self.parse_token(cur_token);
            if parse_result.is_err() {
                return Err(parse_result.err().unwrap());
            }
            
            let parse_result = parse_result?;
            if !parse_result.is_none() {
                parsed_statements.push(parse_result.unwrap());
            }
        }

        Ok(parsed_statements)
    }
    
    fn parse_token(&mut self, token: &Token) -> Result<Option<Statement>, String> {
        if self.statement_type.is_none() {
            let parse_result = self.handle_expecting_header_state(token);
            if parse_result.is_err() {
                return Err(parse_result.err().unwrap())
            }
        } else {
            let parse_result = self.handle_token(token);
            if parse_result.is_err() {
                return Err(parse_result.err().unwrap())
            }
            return parse_result;
        }
        
        Ok(None)
    }

    fn handle_expecting_header_state(&mut self, cur_token: &Token) -> Result<(), String>{
        if
            cur_token.token_type == TokenType::SEPARATOR ||
            cur_token.token_type == TokenType::LINEBREAK
        {
            return Ok(()); // separators and line breaks between statements are ignored
        }
        
        let new_statement_type = Self::convert_token_type_to_statement_type(cur_token.token_type);
        if new_statement_type.is_none() {
            return Err(String::from("Expected statement start"));
        }

        self.statement_type = new_statement_type;
        self.statement_data = Self::convert_token_data_to_statement_data(&cur_token.data);
        self.statement_line_number = cur_token.line_number;
        self.statement_line_position = cur_token.line_position;
        self.parsed_token_count = 1;

        if cur_token.token_type == TokenType::COMMENT {
            self.next_expected_token = TokenType::LINEBREAK;
        } else {
            self.next_expected_token = TokenType::SEPARATOR;
        }

        Ok(())
    }
    
    fn handle_token(&mut self, token: &Token) -> Result<Option<Statement>, String>{
        match self.statement_type {
            Some(StatementType::COMMENT) => self.parse_comment_statement(token),
            Some(StatementType::MTLLIB) => self.parse_single_string_statement(token),
            Some(StatementType::OBJECT) => self.parse_single_string_statement(token),
            Some(StatementType::VERTEX) => self.parse_number_statement(token, 3),
            Some(StatementType::NORMAL) => self.parse_number_statement(token, 3),
            Some(StatementType::TEXCOORD) => self.parse_number_statement(token, 2),
            Some(StatementType::USEMTL) => self.parse_single_string_statement(token),
            Some(StatementType::FACE) => self.parse_face_statement(token),
            Some(StatementType::ILLUM) => self.parse_number_statement(token, 1),
            _ => Ok(None)
        }
    }
    
    fn parse_comment_statement(&mut self, token: &Token) -> Result<Option<Statement>, String>{
        if token.token_type != TokenType::LINEBREAK {
            return Err(Self::get_unexpected_token_error(token));
        }
        
        self.parsed_token_count += 1;
        Ok(Some(self.extract_statement()))
    }
    
    fn parse_single_string_statement(&mut self, token: &Token) -> Result<Option<Statement>, String> {
        if self.next_expected_token == TokenType::SEPARATOR && token.token_type == TokenType::SEPARATOR {
            self.next_expected_token = TokenType::STRING;
            self.parsed_token_count += 1;
            Ok(None)
        } else if self.next_expected_token == TokenType::STRING && token.token_type == TokenType::STRING {
            self.statement_data = Self::convert_token_data_to_statement_data(&token.data);
            self.next_expected_token = TokenType::LINEBREAK;
            self.parsed_token_count += 1;
            Ok(None)
        } else if self.next_expected_token == TokenType::LINEBREAK && token.token_type == TokenType::LINEBREAK {
            self.parsed_token_count += 1;
            Ok(Some(self.extract_statement()))
        } else {
            Err(Self::get_unexpected_token_error(token))
        }
    }
    
    fn parse_number_statement(&mut self, token: &Token, expected_number_count: u64) -> Result<Option<Statement>, String> {
        let tokens_until_line_break = 1 + (expected_number_count * 2);

        if self.next_expected_token == TokenType::SEPARATOR && token.token_type == TokenType::SEPARATOR {
            self.next_expected_token = TokenType::NUMBER;
            
            self.parsed_token_count += 1;
            return Ok(None);
        } else if self.next_expected_token == TokenType::NUMBER && token.token_type == TokenType::NUMBER {
            if let TokenDataType::Number(x) = token.data {
                self.data_buffer.push(x);
            } else {
                return Err(String::from("Number token did not have a number as data"));
            }
            
            self.parsed_token_count += 1;
            
            if self.parsed_token_count >= tokens_until_line_break {
                self.next_expected_token = TokenType::LINEBREAK;
            } else {
                self.next_expected_token = TokenType::SEPARATOR;
            }
            
            return Ok(None);
        } else if self.next_expected_token == TokenType::LINEBREAK && token.token_type == TokenType::LINEBREAK {
            if expected_number_count == 3 && self.data_buffer.len() == 3 {
                self.statement_data = StatementDataType::Number3D(
                    self.data_buffer[0],
                    self.data_buffer[1],
                    self.data_buffer[2]
                );

                self.parsed_token_count += 1;
                return Ok(Some(self.extract_statement()));
            } else if expected_number_count == 2 && self.data_buffer.len() == 2 {
                self.statement_data = StatementDataType::Number2D(
                    self.data_buffer[0],
                    self.data_buffer[1]
                );
                
                self.parsed_token_count += 1;
                return Ok(Some(self.extract_statement()));
            } else if expected_number_count == 1 && self.data_buffer.len() == 1 {
                self.statement_data = StatementDataType::Number(self.data_buffer[0]);
                
                self.parsed_token_count += 1;
                return Ok(Some(self.extract_statement()));
            }
        }
        
        Err(
            format!(
                "Unexpected token. Expected \"{}\" but found \"{}\"",
                TokenType::SEPARATOR,
                TokenType::LINEBREAK
            )
        )
    }
    
    fn parse_face_statement(&mut self, token: &Token) -> Result<Option<Statement>, String> {
        if self.is_expected_token(token, TokenType::SEPARATOR) {
            self.next_expected_token = TokenType::POLYGON;
            
            self.parsed_token_count += 1;
            Ok(None)
        } else if self.is_expected_token(token, TokenType::POLYGON) {
            if let TokenDataType::VertexPTN(x, y, z) = token.data {
                self.index_buffer.push(x);
                self.index_buffer.push(y);
                self.index_buffer.push(z);
            } else {
                return Err(String::from("Expected token data to be VertexPNT"));
            }

            self.parsed_token_count += 1;
            if self.parsed_token_count >= 7 {
                self.next_expected_token = TokenType::LINEBREAK;
            } else {
                self.next_expected_token = TokenType::SEPARATOR;
            }

            Ok(None)
        } else if self.is_expected_token(token, TokenType::LINEBREAK) {
            if self.index_buffer.len() != 9 {
                return Err(String::from("Expected face statement to have 9 indices"));
            }
            self.statement_data = StatementDataType::FacePTN(
                self.index_buffer[0],
                self.index_buffer[1],
                self.index_buffer[2],
                self.index_buffer[3],
                self.index_buffer[4],
                self.index_buffer[5],
                self.index_buffer[6],
                self.index_buffer[7],
                self.index_buffer[8]
            );
            
            self.parsed_token_count += 1;
            Ok(Some(self.extract_statement()))
        } else {
            Err(Self::get_unexpected_token_error(token))
        }
    }
    
    fn is_expected_token(&self, token: &Token, expected_type: TokenType) -> bool {
        self.next_expected_token == token.token_type && token.token_type == expected_type
    }

    fn get_unexpected_token_error(token: &Token) -> String {
        String::from(format!("Unexpected token: {}", token.token_type))
    }

    fn convert_token_data_to_statement_data(token_data: &TokenDataType) -> StatementDataType {
        match token_data {
            TokenDataType::String(s) => StatementDataType::String(String::from(s)),
            TokenDataType::None() => StatementDataType::None(),
            _ => StatementDataType::None()
        }
    }
    
    fn convert_token_type_to_statement_type(token_type: TokenType) -> Option<StatementType> {
        match token_type {
            TokenType::COMMENT => Some(StatementType::COMMENT),
            TokenType::MTLLIB => Some(StatementType::MTLLIB),
            TokenType::OBJECT => Some(StatementType::OBJECT),
            TokenType::VERTEX => Some(StatementType::VERTEX),
            TokenType::NORMAL => Some(StatementType::NORMAL),
            TokenType::TEXCOORD => Some(StatementType::TEXCOORD),
            TokenType::USEMTL => Some(StatementType::USEMTL),
            TokenType::FACE => Some(StatementType::FACE),
            TokenType::ILLUM => Some(StatementType::ILLUM),
            _ => None
        }
    }
    
    fn extract_statement(&mut self) -> Statement {
        let statement = Statement {
            statement_type: self.statement_type.expect("Statement to be set when extracting statement"),
            data: self.statement_data.clone(),
            line_number: self.statement_line_number,
            line_position: self.statement_line_position,
        };
        
        self.reset_state();
        
        statement
    }
    
    fn reset_state(&mut self) {
        self.statement_type = None;
        self.statement_data = StatementDataType::None();
        self.statement_line_number = 0;
        self.statement_line_position = 0;
        self.parsed_token_count = 0;
        self.data_buffer = Vec::new();
        self.index_buffer = Vec::new();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parser_parses_comment_statement() {
        // # This is a comment\n
        parser_parses_tokens_into_statements(
            &vec![
                Token::from(TokenType::COMMENT, TokenDataType::String(String::from("# This is a comment")), 1, 0),
                Token::from(TokenType::LINEBREAK, TokenDataType::String(String::from("\n")), 1, 0),
            ],
            &vec![
                Statement::from(StatementType::COMMENT, StatementDataType::String(String::from("# This is a comment")), 1, 0),
            ]
        );
    }

    #[test]
    fn parser_parses_mtllib_statement() {
        // mtllib file.mtl\n
        parser_parses_tokens_into_statements(
            &vec![
                Token::from(TokenType::MTLLIB, TokenDataType::None(), 1, 0),
                Token::from(TokenType::SEPARATOR, TokenDataType::None(), 1, 0),
                Token::from(TokenType::STRING, TokenDataType::String(String::from("file.mtl")), 1, 0),
                Token::from(TokenType::LINEBREAK, TokenDataType::String(String::from("\n")), 1, 0),
            ],
            &vec![
                Statement::from(StatementType::MTLLIB, StatementDataType::String(String::from("file.mtl")), 1, 0),
            ]
        );
    }
    
    #[test]
    fn parser_parses_object_statement() {
        // o object_name\n
        parser_parses_tokens_into_statements(
            &vec![
                Token::from(TokenType::OBJECT, TokenDataType::None(), 1, 0),
                Token::from(TokenType::SEPARATOR, TokenDataType::None(), 1, 0),
                Token::from(TokenType::STRING, TokenDataType::String(String::from("object_name")), 1, 0),
                Token::from(TokenType::LINEBREAK, TokenDataType::String(String::from("\n")), 1, 0),
            ],
            &vec![
                Statement::from(StatementType::OBJECT, StatementDataType::String(String::from("object_name")), 1, 0),
            ]
        );
    }
    
    #[test]
    fn parser_parses_vertex_statement() {
        // v 1.0 2.0 3.0\n
        parser_parses_tokens_into_statements(
            &vec![
                Token::from(TokenType::VERTEX, TokenDataType::None(), 1, 0),
                Token::from(TokenType::SEPARATOR, TokenDataType::None(), 1, 0),
                Token::from(TokenType::NUMBER, TokenDataType::Number(1.0), 1, 0),
                Token::from(TokenType::SEPARATOR, TokenDataType::None(), 1, 0),
                Token::from(TokenType::NUMBER, TokenDataType::Number(2.0), 1, 0),
                Token::from(TokenType::SEPARATOR, TokenDataType::None(), 1, 0),
                Token::from(TokenType::NUMBER, TokenDataType::Number(3.0), 1, 0),
                Token::from(TokenType::LINEBREAK, TokenDataType::String(String::from("\n")), 1, 0),
            ],
            &vec![
                Statement::from(StatementType::VERTEX, StatementDataType::Number3D(1.0, 2.0, 3.0), 1, 0),
            ]
        );
    }
    
    #[test]
    fn parser_parses_normal_statement() {
        // vn 0.707 0.0 0.707\n
        parser_parses_tokens_into_statements(
            &vec![
                Token::from(TokenType::NORMAL, TokenDataType::None(), 1, 0),
                Token::from(TokenType::SEPARATOR, TokenDataType::None(), 1, 0),
                Token::from(TokenType::NUMBER, TokenDataType::Number(0.707), 1, 0),
                Token::from(TokenType::SEPARATOR, TokenDataType::None(), 1, 0),
                Token::from(TokenType::NUMBER, TokenDataType::Number(0.0), 1, 0),
                Token::from(TokenType::SEPARATOR, TokenDataType::None(), 1, 0),
                Token::from(TokenType::NUMBER, TokenDataType::Number(0.707), 1, 0),
                Token::from(TokenType::LINEBREAK, TokenDataType::String(String::from("\n")), 1, 0),
            ],
            &vec![
                Statement::from(StatementType::NORMAL, StatementDataType::Number3D(0.707, 0.0, 0.707), 1, 0),
            ]
        );
    }
    
    #[test]
    fn parser_parses_texcoord_statement() {
        // vt 0.75 0.25\n
        parser_parses_tokens_into_statements(
            &vec![
                Token::from(TokenType::TEXCOORD, TokenDataType::None(), 1, 0),
                Token::from(TokenType::SEPARATOR, TokenDataType::None(), 1, 0),
                Token::from(TokenType::NUMBER, TokenDataType::Number(0.75), 1, 0),
                Token::from(TokenType::SEPARATOR, TokenDataType::None(), 1, 0),
                Token::from(TokenType::NUMBER, TokenDataType::Number(0.25), 1, 0),
                Token::from(TokenType::LINEBREAK, TokenDataType::String(String::from("\n")), 1, 0),
            ],
            &vec![
                Statement::from(StatementType::TEXCOORD, StatementDataType::Number2D(0.75, 0.25), 1, 0),
            ]
        );
    }
    
    #[test]
    fn parser_parses_usemtl_statement() {
        // usemtl name\n
        parser_parses_tokens_into_statements(
            &vec![
                Token::from(TokenType::USEMTL, TokenDataType::None(), 1, 0),
                Token::from(TokenType::SEPARATOR, TokenDataType::None(), 1, 0),
                Token::from(TokenType::STRING, TokenDataType::String(String::from("name")), 1, 0),
                Token::from(TokenType::LINEBREAK, TokenDataType::String(String::from("\n")), 1, 0),
            ],
            &vec![
                Statement::from(StatementType::USEMTL, StatementDataType::String(String::from("name")), 1, 0),
            ]
        );
    }
    
    #[test]
    fn parser_parses_face_statement() {
        // f 1/2/3 4/5/6 7/8/9\n
        parser_parses_tokens_into_statements(
            &vec![
                Token::from(TokenType::FACE, TokenDataType::None(), 1, 0),
                Token::from(TokenType::SEPARATOR, TokenDataType::None(), 1, 0),
                Token::from(TokenType::POLYGON, TokenDataType::VertexPTN(1, 2, 3), 1, 0),
                Token::from(TokenType::SEPARATOR, TokenDataType::None(), 1, 0),
                Token::from(TokenType::POLYGON, TokenDataType::VertexPTN(4, 5, 6), 1, 0),
                Token::from(TokenType::SEPARATOR, TokenDataType::None(), 1, 0),
                Token::from(TokenType::POLYGON, TokenDataType::VertexPTN(7, 8, 9), 1, 0),
                Token::from(TokenType::LINEBREAK, TokenDataType::String(String::from("\n")), 1, 0),
            ],
            &vec![
                Statement::from(StatementType::FACE, StatementDataType::FacePTN(1, 2, 3, 4, 5, 6, 7, 8, 9), 1, 0),
            ]
        );
    }
    
    #[test]
    fn parser_parses_illum_statement() {
        // s 1\n
        parser_parses_tokens_into_statements(
            &vec![
                Token::from(TokenType::ILLUM, TokenDataType::None(), 1, 0),
                Token::from(TokenType::SEPARATOR, TokenDataType::None(), 1, 0),
                Token::from(TokenType::NUMBER, TokenDataType::Number(1.0), 1, 0),
                Token::from(TokenType::LINEBREAK, TokenDataType::String(String::from("\n")), 1, 0),
            ],
            &vec![
                Statement::from(StatementType::ILLUM, StatementDataType::Number(1.0), 1, 0),
            ]
        );
    }
    
    #[test]
    fn parser_parses_multiple_statements() {
        // v 1.0 2.0 3.0\n
        // vn 0.707 0.0 0.707\n
        parser_parses_tokens_into_statements(
            &vec![
                Token::from(TokenType::VERTEX, TokenDataType::None(), 1, 0),
                Token::from(TokenType::SEPARATOR, TokenDataType::None(), 1, 0),
                Token::from(TokenType::NUMBER, TokenDataType::Number(1.0), 1, 0),
                Token::from(TokenType::SEPARATOR, TokenDataType::None(), 1, 0),
                Token::from(TokenType::NUMBER, TokenDataType::Number(2.0), 1, 0),
                Token::from(TokenType::SEPARATOR, TokenDataType::None(), 1, 0),
                Token::from(TokenType::NUMBER, TokenDataType::Number(3.0), 1, 0),
                Token::from(TokenType::LINEBREAK, TokenDataType::String(String::from("\n")), 1, 0),
                
                Token::from(TokenType::NORMAL, TokenDataType::None(), 1, 0),
                Token::from(TokenType::SEPARATOR, TokenDataType::None(), 1, 0),
                Token::from(TokenType::NUMBER, TokenDataType::Number(0.707), 1, 0),
                Token::from(TokenType::SEPARATOR, TokenDataType::None(), 1, 0),
                Token::from(TokenType::NUMBER, TokenDataType::Number(0.0), 1, 0),
                Token::from(TokenType::SEPARATOR, TokenDataType::None(), 1, 0),
                Token::from(TokenType::NUMBER, TokenDataType::Number(0.707), 1, 0),
                Token::from(TokenType::LINEBREAK, TokenDataType::String(String::from("\n")), 1, 0),
            ],
            &vec![
                Statement::from(StatementType::VERTEX, StatementDataType::Number3D(1.0, 2.0, 3.0), 1, 0),
                Statement::from(StatementType::NORMAL, StatementDataType::Number3D(0.707, 0.0, 0.707), 1, 0),
            ]
        );
    }

    fn parser_parses_tokens_into_statements(
        input_tokens: &Vec<Token>,
        expected_statements: &Vec<Statement>
    ) {
        let parser = Parser::new();

        let result = parser.parse_tokens(input_tokens);
        
        assert!(
            result.is_ok(),
            "Parser returns okay when parsing valid token sequence", 
        );

        assert_statement_vectors_are_equal(expected_statements, &result.unwrap());
    }

    fn assert_statement_vectors_are_equal(
        expected: &Vec<Statement>,
        actual: &Vec<Statement>,
    ) {
        let expected_vector_length = expected.len();

        assert_eq!(
            expected_vector_length,
            actual.len(),
            "Parser returns {expected_vector_length} tokens"
        );

        for i in 0..expected_vector_length {
            let expected_statement = &expected[i];
            let actual_statement = &actual[i];

            assert_eq!(
                expected_statement.statement_type,
                actual_statement.statement_type,
                "Parser returns the correct type for statement {i}"
            );

            assert_eq!(
                expected_statement.data,
                actual_statement.data,
                "Parser returns the correct data for statement {i}"
            );

            assert_eq!(
                expected_statement.line_number,
                actual_statement.line_number,
                "Parser returns the correct line number for statement {i}"
            );

            assert_eq!(
                expected_statement.line_position,
                actual_statement.line_position,
                "Parser returns the correct line position for statement {i}"
            );
        }
    }
}