use std::collections::VecDeque;
use std::io::{Read};
use std::str::FromStr;
use crate::token;
use token::{Token, TokenType, TokenDataType};
use crate::nan_safe_float::Float;

#[derive(PartialEq)]
enum LexerState {
    Initial,
    Token,
    LineBreak,
    Separator,
    Comment,
}

struct Lexer {
    char_buffer: String,
    char_position: u64,
    line_number: u64,
    state: LexerState,
}

impl Default for Lexer {
    fn default() -> Self {
        Lexer {
            char_buffer: String::new(),
            char_position: 0,
            line_number: 1,
            state: LexerState::Initial,
        }
    }
}

impl Lexer {
    fn new() -> Self {
        Default::default()
    }

    fn lex_tokens<R: Read>(&mut self, stream: &mut R) -> Vec<Token> {
        let mut lexed_tokens = Vec::new();

        loop {
            let cur_char = Lexer::advance_char(stream);
            if cur_char.is_none() {
                self.process_char_buffer(&mut lexed_tokens);
                break;
            }
            let cur_char = cur_char.unwrap();

            let next_state = self.check_for_state_transition(cur_char);
            if !next_state.is_none() {
                self.process_char_buffer(&mut lexed_tokens);
                self.state = next_state.unwrap();
            }

            self.save_char(cur_char);
        }

        lexed_tokens
    }

    fn advance_char<R: Read>(stream: &mut R) -> Option<char> {
        let mut buffer = [0; 1];

        let result = stream.read(&mut buffer);
        if result.is_err() || result.unwrap() == 0 {
            return None
        }

        Some(char::from(buffer[0]))
    }

    fn check_for_state_transition(&mut self, cur_char: char) -> Option<LexerState> {
        let is_n_line_ending = cur_char == '\n';
        let is_line_ending = cur_char == '\n' || cur_char == '\r';
        let is_whitespace = cur_char.is_whitespace() && !is_line_ending;
        let is_comment = cur_char == '#';
        let is_normal = !(is_line_ending || is_whitespace || is_comment);
        let has_nr_line_ending = self.char_buffer == "\n\r";

        if is_line_ending && self.state != LexerState::LineBreak {
            return Some(LexerState::LineBreak); //fake a state transition by returning the current state
        } else if is_n_line_ending && self.state == LexerState::LineBreak {
            return Some(LexerState::LineBreak); //fake a state transition by returning the current state
        } else if is_line_ending && has_nr_line_ending {
            return Some(LexerState::LineBreak); //fake a state transition by returning the current state
        } else if is_line_ending && self.state == LexerState::Comment {
            return Some(LexerState::LineBreak);
        } else if is_whitespace && self.state != LexerState::Separator && self.state != LexerState::Comment {
            return Some(LexerState::Separator);
        } else if is_comment && self.state != LexerState::Comment {
            return Some(LexerState::Comment);
        } else if is_normal && self.state != LexerState::Token && self.state != LexerState::Comment {
            return Some(LexerState::Token);
        }

        None
    }

    fn save_char(&mut self, cur_char: char) {
        self.char_buffer.push(cur_char);
        self.char_position += 1;
    }

    fn process_char_buffer(&mut self, lexed_tokens: &mut Vec<Token>) {
        if self.char_buffer.len() == 0 {
            return;
        }

        let char_buffer = self.char_buffer.clone();
        self.char_buffer = String::new();
        let char_pos = self.char_position - (char_buffer.len() as u64) + 1;
        let mut new_token: Option<Token> = None;

        if self.state == LexerState::Comment {
            new_token = Some(
                Token::from(
                    TokenType::COMMENT,
                    TokenDataType::String(String::from(char_buffer.clone())),
                    self.line_number,
                    char_pos
                )
            );
        } else if self.state == LexerState::LineBreak {
            new_token = Some(
                Token::from(
                    TokenType::LINEBREAK,
                    TokenDataType::String(String::from(char_buffer.clone())),
                    self.line_number,
                    char_pos
                )
            );

            self.char_position = 0;
            self.line_number += 1;
        } else if self.state == LexerState::Separator {
            new_token = Some(
                Token::from(
                    TokenType::SEPARATOR,
                    TokenDataType::None(),
                    self.line_number,
                    char_pos
                )
            )
        }

        if new_token.is_none() {
            let token_type = TokenType::from_str(char_buffer.clone().as_str());
            if !token_type.is_none() {
                new_token = Some(
                    Token::from(
                        token_type.unwrap(),
                        TokenDataType::None(),
                        self.line_number,
                        char_pos
                    )
                );
            }
        }

        if new_token.is_none() {
            let parse_float_result = f64::from_str(char_buffer.clone().as_str());
            if !parse_float_result.is_err() {
                let parse_float_result = Float::new(parse_float_result.unwrap());
                if !parse_float_result.is_err() {
                    new_token = Some(
                        Token::from(
                            TokenType::NUMBER,
                            TokenDataType::Number(parse_float_result.unwrap()),
                            self.line_number,
                            char_pos
                        )
                    );
                }
            }
        }

        if new_token.is_none() {
            let lex_polygon_result = Lexer::lex_polygon(char_buffer.clone().as_str());
            if !lex_polygon_result.is_none() {
                new_token = Some(
                    Token::from(
                        TokenType::POLYGON,
                        lex_polygon_result.unwrap(),
                        self.line_number,
                        char_pos,
                    )
                );
            }
        }

        if new_token.is_none() {
            new_token = Some(
                Token::from(
                    TokenType::STRING,
                    TokenDataType::String(String::from(char_buffer.clone().as_str())),
                    self.line_number,
                    char_pos,
                )
            );
        }

        let new_token = new_token.expect("Lexer to lex a token");
        lexed_tokens.push(new_token);
    }

    fn lex_polygon(text: &str) -> Option<TokenDataType> {
        let mut chars = VecDeque::from_iter(text.chars());
        let mut buffer = String::new();
        let mut data: Vec<u64> = Vec::new();
        let mut divider_count = 0;

        while chars.len() > 0 {
            let cur_char = chars.pop_front()?;
            if cur_char != '/' {
                buffer.push(cur_char);
            } else {
                if divider_count >= 2 {
                    return None;
                }
                divider_count += 1;
                if buffer.len() == 0 {
                    data.push(0); //TODO: wfo indices are 1 based ... so I should be able to do this?
                } else {
                    let int_parse_result = u64::from_str(&buffer);
                    if int_parse_result.is_err() {
                        return None
                    }
                    data.push(int_parse_result.unwrap());
                    buffer = String::new();
                }
            }
        }
        if buffer.len() == 0 {
            data.push(0); //TODO: wfo indices are 1 based ... so I should be able to do this?
        } else {
            let int_parse_result = u64::from_str(&buffer);
            if int_parse_result.is_err() {
                return None
            }
            data.push(int_parse_result.unwrap());
        }

        if data.len() != 3 {
            None
        } else {
            Some(TokenDataType::VertexPTN(data[0], data[1], data[2]))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::f;
    use super::*;

    #[test]
    fn lexer_lexes_comment() {
        test_lexer_lexes_single_token(
            &vec![Token::from(TokenType::COMMENT, TokenDataType::String(String::from("# This is a comment")), 1, 1)],
            "# This is a comment"
        );
    }

    #[test]
    fn lexer_lexes_mtllib() {
        test_lexer_lexes_single_token(
            &vec![Token::from(TokenType::MTLLIB, TokenDataType::None(), 1, 1)],
            "mtllib"
        );
    }

    #[test]
    fn lexer_lexes_object() {
        test_lexer_lexes_single_token(
            &vec![Token::from(TokenType::OBJECT, TokenDataType::None(), 1, 1)],
            "o"
        );
    }

    #[test]
    fn lexer_lexes_vertex() {
        test_lexer_lexes_single_token(
            &vec![Token::from(TokenType::VERTEX, TokenDataType::None(), 1, 1)],
            "v"
        );
    }

    #[test]
    fn lexer_lexes_normal() {
        test_lexer_lexes_single_token(
            &vec![Token::from(TokenType::NORMAL, TokenDataType::None(), 1, 1)],
            "vn"
        );
    }

    #[test]
    fn lexer_lexes_texcoord() {
        test_lexer_lexes_single_token(
            &vec![Token::from(TokenType::TEXCOORD, TokenDataType::None(), 1, 1)],
            "vt"
        );
    }

    #[test]
    fn lexer_lexes_usemtl() {
        test_lexer_lexes_single_token(
            &vec![Token::from(TokenType::USEMTL, TokenDataType::None(), 1, 1)],
            "usemtl"
        );
    }

    #[test]
    fn lexer_lexes_face() {
        test_lexer_lexes_single_token(
            &vec![Token::from(TokenType::FACE, TokenDataType::None(), 1, 1)],
            "f"
        );
    }

    #[test]
    fn lexer_lexes_illum() {
        test_lexer_lexes_single_token(
            &vec![Token::from(TokenType::ILLUM, TokenDataType::None(), 1, 1)],
            "s"
        );
    }

    #[test]
    fn lexer_lexes_number() {
        test_lexer_lexes_single_token(
            &vec![Token::from(TokenType::NUMBER, TokenDataType::Number(f!(1.0)), 1, 1)],
            "1.0"
        );
    }

    #[test]
    fn lexer_lexes_polygon() {
        test_lexer_lexes_single_token(
            &vec![Token::from(TokenType::POLYGON, TokenDataType::VertexPTN(1, 2, 3), 1, 1)],
            "1/2/3"
        );
    }

    #[test]
    fn lexer_lexes_polygon_without_texcoord_indices() {
        test_lexer_lexes_single_token(
            &vec![Token::from(TokenType::POLYGON, TokenDataType::VertexPTN(1, 0, 2), 1, 1)],
            "1//2"
        );
    }

    #[test]
    fn lexer_lexes_polygon_without_normal_indices() {
        test_lexer_lexes_single_token(
            &vec![Token::from(TokenType::POLYGON, TokenDataType::VertexPTN(1, 2, 0), 1, 1)],
            "1/2/"
        );
    }

    #[test]
    fn lexer_lexes_string() {
        test_lexer_lexes_single_token(
            &vec![Token::from(TokenType::STRING, TokenDataType::String(String::from("asdf")), 1, 1)],
            "asdf"
        );
    }

    #[test]
    fn lexer_lexes_separator() {
        test_lexer_lexes_single_token(
            &vec![Token::from(TokenType::SEPARATOR, TokenDataType::None(), 1, 1)],
            " "
        );
    }

    #[test]
    fn lexer_lexes_separator_with_multiple_spaces() {
        test_lexer_lexes_single_token(
            &vec![Token::from(TokenType::SEPARATOR, TokenDataType::None(), 1, 1)],
            " \t "
        );
    }

    #[test]
    fn lexer_lexes_unix_line_break() {
        test_lexer_lexes_single_token(
            &vec![Token::from(TokenType::LINEBREAK, TokenDataType::String(String::from("\n")), 1, 1)],
            "\n"
        );
    }

    #[test]
    fn lexer_lexes_macos_line_break() {
        test_lexer_lexes_single_token(
            &vec![Token::from(TokenType::LINEBREAK, TokenDataType::String(String::from("\r")), 1, 1)],
            "\r"
        );
    }

    #[test]
    fn lexer_lexes_windows_line_break() {
        test_lexer_lexes_single_token(
            &vec![Token::from(TokenType::LINEBREAK, TokenDataType::String(String::from("\n\r")), 1, 1)],
            "\n\r"
        );
    }

    fn test_lexer_lexes_single_token(expected_result: &Vec<Token>, text: &str) {
        let mut lexer = Lexer::new();

        let result = lexer.lex_tokens(&mut text.as_bytes());

        assert_token_vectors_are_equal(
            &expected_result,
            &result
        );
    }

    #[test]
    fn lexer_lexes_multiple_line_endings() {
        let test_data = "\r\n\r\n\n\r\n\n\r\r";  // very unlikely but it should handle it
        let expected_tokens = vec!(
            Token::from(TokenType::LINEBREAK, TokenDataType::String(String::from("\r")), 1, 1),
            Token::from(TokenType::LINEBREAK, TokenDataType::String(String::from("\n\r")), 2, 1),
            Token::from(TokenType::LINEBREAK, TokenDataType::String(String::from("\n")), 3, 1),
            Token::from(TokenType::LINEBREAK, TokenDataType::String(String::from("\n\r")), 4, 1),
            Token::from(TokenType::LINEBREAK, TokenDataType::String(String::from("\n")), 5, 1),
            Token::from(TokenType::LINEBREAK, TokenDataType::String(String::from("\n\r")), 6, 1),
            Token::from(TokenType::LINEBREAK, TokenDataType::String(String::from("\r")), 7, 1),
        );

        let mut lexer = Lexer::new();

        let result = lexer.lex_tokens(&mut test_data.as_bytes());

        assert_token_vectors_are_equal(
            &expected_tokens,
            &result
        );
    }

    #[test]
    fn lexer_lexes_multiple_tokens_from_the_same_line() {
        let test_data = "v 0.00 1.00 2.00\n";
        let expected_tokens = vec!(
            Token::from(TokenType::VERTEX, TokenDataType::None(), 1, 1),
            Token::from(TokenType::SEPARATOR, TokenDataType::None(), 1, 2),
            Token::from(TokenType::NUMBER, TokenDataType::Number(f!(0.0)), 1, 3),
            Token::from(TokenType::SEPARATOR, TokenDataType::None(), 1, 7),
            Token::from(TokenType::NUMBER, TokenDataType::Number(f!(1.0)), 1, 8),
            Token::from(TokenType::SEPARATOR, TokenDataType::None(), 1, 12),
            Token::from(TokenType::NUMBER, TokenDataType::Number(f!(2.0)), 1, 13),
            Token::from(TokenType::LINEBREAK, TokenDataType::String(String::from("\n")), 1, 17),
        );

        let mut lexer = Lexer::new();

        let result = lexer.lex_tokens(&mut test_data.as_bytes());

        assert_token_vectors_are_equal(
            &expected_tokens,
            &result
        );
    }

    #[test]
    fn lexer_lexes_multiple_tokens_from_multiple_lines() {
        let test_data = "# First line comment\nv 0.00 1.00 2.00\nusemtl some-material\n\ns 1\n";
        let expected_tokens = vec!(
            Token::from(TokenType::COMMENT, TokenDataType::String(String::from("# First line comment")), 1, 1),
            Token::from(TokenType::LINEBREAK, TokenDataType::String(String::from("\n")), 1, 21),

            Token::from(TokenType::VERTEX, TokenDataType::None(), 2, 1),
            Token::from(TokenType::SEPARATOR, TokenDataType::None(), 2, 2),
            Token::from(TokenType::NUMBER, TokenDataType::Number(f!(0.0)), 2, 3),
            Token::from(TokenType::SEPARATOR, TokenDataType::None(), 2, 7),
            Token::from(TokenType::NUMBER, TokenDataType::Number(f!(1.0)), 2, 8),
            Token::from(TokenType::SEPARATOR, TokenDataType::None(), 2, 12),
            Token::from(TokenType::NUMBER, TokenDataType::Number(f!(2.0)), 2, 13),
            Token::from(TokenType::LINEBREAK, TokenDataType::String(String::from("\n")), 2, 17),

            Token::from(TokenType::USEMTL, TokenDataType::None(), 3, 1),
            Token::from(TokenType::SEPARATOR, TokenDataType::None(), 3, 7),
            Token::from(TokenType::STRING, TokenDataType::String(String::from("some-material")), 3, 8),
            Token::from(TokenType::LINEBREAK, TokenDataType::String(String::from("\n")), 3, 21),

            Token::from(TokenType::LINEBREAK, TokenDataType::String(String::from("\n")), 4, 1),

            Token::from(TokenType::ILLUM, TokenDataType::None(), 5, 1),
            Token::from(TokenType::SEPARATOR, TokenDataType::None(), 5, 2),
            Token::from(TokenType::NUMBER, TokenDataType::Number(f!(1.0)), 5, 3),
            Token::from(TokenType::LINEBREAK, TokenDataType::String(String::from("\n")), 5, 4),
        );

        let mut lexer = Lexer::new();
        let result = lexer.lex_tokens(&mut test_data.as_bytes());

        assert_token_vectors_are_equal(
            &expected_tokens,
            &result
        );
    }

    fn assert_token_vectors_are_equal(expected_result: &Vec<Token>, actual_result: &Vec<Token>) {
        let expected_vector_length = expected_result.len();

        assert_eq!(
            expected_vector_length,
            actual_result.len(),
            "Lexer returns {expected_vector_length} tokens"
        );

        for i in 0..expected_vector_length {
            let expected_token = &expected_result[i];
            let actual_token = &actual_result[i];

            assert_eq!(
                expected_token.token_type,
                actual_token.token_type,
                "Lexer returns the correct type for token {i}"
            );

            assert_eq!(
                expected_token.data,
                actual_token.data,
                "Lexer returns the correct data for token {i}"
            );

            assert_eq!(
                expected_token.line_number,
                actual_token.line_number,
                "Lexer returns the correct line number for token {i}"
            );

            assert_eq!(
                expected_token.line_position,
                actual_token.line_position,
                "Lexer returns the correct line position for token {i}"
            );
        }
    }
}
