#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum Token {
    Text(String),

    Identifier(String),
    StringLiteral(String),

    LegacyHelper, // function.

    BlockOpen, // {{{, <!--
    BlockClose, // }}}, -->

    If, // if, IF
    Else, // else, ELSE
    Iter, // each, BEGIN
    End, // end, END, ENDIF

    Bang, // !
    LeftParen, // (
    RightParen, // )
    Comma, // ,

    RawOpen, // {{
    RawClose, // }}
    EscapedOpen, // {
    EscapedClose, // }
}

impl ToString for Token {
    fn to_string(&self) -> String {
        match self {
            &Token::Text(ref val) => val.to_string(),

            &Token::Identifier(ref val) => val.to_string(),
            &Token::StringLiteral(ref val) => format!("\"{}\"", val.to_string()),

            &Token::LegacyHelper => "function.".to_string(), // function.

            &Token::BlockOpen => String::new(), // {{{, <!--
            &Token::BlockClose => String::new(), // }}}, -->

            &Token::If => String::new(), // if, IF
            &Token::Else => String::new(), // else, ELSE
            &Token::Iter => String::new(), // each, BEGIN
            &Token::End => String::new(), // end, END, ENDIF

            &Token::Bang => "!".to_string(), // !
            &Token::LeftParen => "(".to_string(), // (
            &Token::RightParen => ")".to_string(), // )
            &Token::Comma => ",".to_string(), // ,

            &Token::RawOpen => "{{".to_string(), // {{
            &Token::RawClose => "}}".to_string(), // }}
            &Token::EscapedOpen => "{".to_string(), // {
            &Token::EscapedClose => "}".to_string(), // }
        }
    }
}
