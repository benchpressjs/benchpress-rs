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

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct TokenPos {
    pub start: usize,
    pub end: usize,

    pub tok: Token,
}
