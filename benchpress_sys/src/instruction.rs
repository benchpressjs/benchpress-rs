use token::{TokenPos, Token};

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum Instruction {
    Text(String),
    Escaped(Vec<Token>),
    Raw(Vec<Token>),
    IfStart(Vec<Token>),
    IterStart(Vec<Token>),
    Else,
    End(Vec<Token>),
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct InstructionPos {
    pub start: usize,
    pub end: usize,

    pub inst: Instruction,
}

impl InstructionPos {
    pub fn get_source(&self, source: &str) -> String {
        source[self.start..self.end].to_string()
    }

    pub fn to_text(&self, source: &str) -> InstructionPos {
        InstructionPos {
            start: self.start,
            end: self.end,
            inst: Instruction::Text(self.get_source(source)),
        }
    }

    pub fn from_text(input: TokenPos) -> Option<InstructionPos> {
        match input {
            TokenPos { start, end, tok: Token::Text(text) } => Some(InstructionPos {
                start,
                end,
                inst: Instruction::Text(text),
            }),
            _ => None,
        }
    }
}

