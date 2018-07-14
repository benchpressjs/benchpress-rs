use token::{Token, TokenPos};
use instruction::{Instruction, InstructionPos};

use std::iter::{Iterator, Peekable};
use itertools::Itertools;

pub fn parse_instructions(_source: &str, tokens: Vec<TokenPos>) -> Vec<InstructionPos> {
    let mut output: Vec<InstructionPos> = vec![];

    let mut iter = tokens.into_iter().peekable();

    while let Some(opener) = iter.next() {
        if let Some(inst_pos) = match opener {
            TokenPos { tok: Token::Text(_), .. } => InstructionPos::from_text(opener),
            TokenPos { tok: Token::BlockOpen, start, .. } => {
                let TokenPos { tok: keyword, .. } = iter.next().unwrap();

                let expr: Vec<Token> = iter.peeking_take_while(|x| match x {
                    TokenPos { tok: Token::BlockClose, .. } => false,
                    _ => true,
                }).map(|TokenPos { tok, .. }| tok).collect();

                let TokenPos { end, .. } = iter.next().unwrap();

                if let Some(inst) = match keyword {
                    Token::If => Some(Instruction::IfStart(expr)),
                    Token::Iter => Some(Instruction::IterStart(expr)),
                    Token::Else => Some(Instruction::Else),
                    Token::End => Some(Instruction::End(expr)),
                    _ => None,
                } {
                    Some(InstructionPos {
                        start,
                        end,
                        inst,
                    })
                } else { None }
            },
            TokenPos { tok: Token::RawOpen, start, .. } | 
            TokenPos { tok: Token::EscapedOpen, start, .. } => {
                let closer = match opener {
                    TokenPos { tok: Token::RawOpen, .. } => Token::RawClose,
                    // TokenPos { tok: Token::EscapedOpen, .. }
                    _ => Token::EscapedClose,
                };

                let expr: Vec<Token> = iter
                    .peeking_take_while(|TokenPos { tok, .. }| tok != &closer)
                    .map(|TokenPos { tok, .. }| tok).collect();

                let TokenPos { end, .. } = iter.next().unwrap();

                let inst = match opener {
                    TokenPos { tok: Token::RawOpen, .. } => Instruction::Raw(expr),
                    // TokenPos { tok: Token::EscapedOpen, .. }
                    _ => Instruction::Escaped(expr),
                };

                Some(InstructionPos {
                    start,
                    end,
                    inst,
                })
            },
            _ => None,
        } {
            output.push(inst_pos);
        }
    }

    output
}

use std::collections::HashSet;

pub fn starts_with(full: &Vec<Token>, part: &Vec<Token>) -> bool {
    if part.len() > full.len() {
        return false;
    }

    for i in 0..part.len() {
        if full[i] != part[i] {
            return false;
        }
    }

    true
}

pub fn fix_extra_tokens(source: &str, input: Vec<InstructionPos>) -> Vec<InstructionPos> {
    let mut remove: HashSet<InstructionPos> = HashSet::new();
    let mut expected_subjects: Vec<Vec<Token>> = Vec::new();

    let mut starts_count: u16 = 0;
    let mut ends_count: u16 = 0;

    // try to find a Close with no corresponding Open
    for index in 0..input.len() {
        let elem = &input[index];

        match elem {
            InstructionPos { inst: Instruction::IfStart(ref subject), .. } | 
            InstructionPos { inst: Instruction::IterStart(ref subject), .. } => {
                expected_subjects.push(subject.clone());
                starts_count += 1;
            },
            InstructionPos { inst: Instruction::End(ref subject), .. } => {
                ends_count += 1;

                if let Some(expected_subject) = expected_subjects.pop() {
                    if subject.len() > 0 && !starts_with(&expected_subject, subject) {
                        remove.insert(elem.clone());
                        expected_subjects.push(expected_subject);
                    } else {
                        // search for an end within close proximity
                        // that has the expected subject
                        for i in (index + 1)..input.len() {
                            let ahead = &input[i];
                            match ahead {
                                InstructionPos { inst: Instruction::IfStart(_), .. } |
                                InstructionPos { inst: Instruction::IterStart(_), .. } => {
                                    break;
                                },
                                InstructionPos { inst: Instruction::End(ref ahead_subject), .. } => {
                                    if ahead_subject.clone() == expected_subject {
                                        // found one ahead, so remove the current one
                                        remove.insert(elem.clone());
                                        expected_subjects.push(expected_subject);

                                        break;
                                    }
                                },
                                _ => (),
                            }
                        }
                    }
                } else {
                    remove.insert(elem.clone());
                }
            },
            _ => (),
        }
    }

    if ends_count > starts_count {
        let mut diff = ends_count - starts_count;

        println!("Found extra token(s):");

        let output: Vec<InstructionPos> = input.into_iter().map(|x| if remove.contains(&x) && diff > 0 {
            println!("{:?}", x);

            diff -= 1;
            x.to_text(source)
        } else { x }).collect();

        println!("These tokens will be passed through as text, but you should remove them to prevent issues in the future.");

        output
    } else {
        input
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    HelperExpression { helper_name: String, args: Vec<Expression> },
    PathExpression { path: Vec<String> },
    StringLiteral { value: String },
    NegativeExpression { expr: Box<Expression> },
}

#[derive(Debug, PartialEq, Clone)]
pub enum Control {
    Text { value: String },
    If { subject: Expression, body: Vec<Control>, alt: Vec<Control> },
    Iter { suffix: u16, subject_raw: String, subject: Expression, body: Vec<Control>, alt: Vec<Control> },
    Escaped { subject: Expression },
    Raw { subject: Expression },
}

fn generate_expression<I>(iter: &mut Peekable<I>, base: Vec<String>, suffix: u16) -> Option<Expression>
where
    I: Iterator<Item = Token>
{
    let one = iter.next();
    let two = match iter.peek() {
        Some(thing) => Some(thing.clone()),
        None => None,
    };

    match (one, two) {
        (Some(Token::Bang), Some(_)) => {
            if let Some(expr) = generate_expression(iter.by_ref(), base, suffix) {
                Some(Expression::NegativeExpression {
                    expr: Box::new(expr),
                })
            } else {
                None
            }
        },
        (Some(Token::Identifier(ref name)), Some(Token::LeftParen)) |
        (Some(Token::LegacyHelper), Some(Token::Identifier(ref name))) => {
            iter.next();
            let mut args: Vec<Expression> = Vec::new();

            loop {
                let (skip, done): (bool, bool) = match iter.peek() {
                    Some(&Token::Comma) => (true, false),
                    Some(&Token::RightParen) => (true, true),
                    Some(_) => (false, false),
                    None => (false, true),
                };

                if skip {
                    iter.next();
                }
                if done {
                    break;
                }

                if let Some(arg) = generate_expression(iter.by_ref(), base.clone(), suffix) {
                    args.push(arg);
                }
            }

            Some(Expression::HelperExpression {
                helper_name: name.to_string(),
                args,
            })
        },
        (Some(Token::StringLiteral(value)), None) |
        (Some(Token::StringLiteral(value)), Some(Token::Comma)) |
        (Some(Token::StringLiteral(value)), Some(Token::RightParen)) => Some(Expression::StringLiteral {
            value
        }),
        (Some(Token::Identifier(value)), None) |
        (Some(Token::Identifier(value)), Some(Token::Comma)) |
        (Some(Token::Identifier(value)), Some(Token::RightParen)) => {
            let path: Vec<String> = paths::split(value);

            Some(Expression::PathExpression { path: paths::resolve(base, path) })
        },
        _ => None,
    }
}

use paths;

// build the tree
pub fn parse_tree<I>(
    source: &str,
    input: &mut I,
    base: Vec<String>,
    suffix: u16,
) -> (Vec<Control>, Option<InstructionPos>)
where
    I: Iterator<Item = InstructionPos>
{
    let mut output: Vec<Control> = Vec::new();

    let mut last: Option<InstructionPos> = None;

    while let Some(inst_pos) = input.next() {
        let InstructionPos { inst, .. } = inst_pos.clone();
        match inst {
            Instruction::Text(value) => output.push(Control::Text { value }),
            Instruction::Escaped(subject)  => if let Some(subject) = generate_expression(
                &mut subject.into_iter().peekable(),
                base.clone(),
                suffix
            ) {
                output.push(Control::Escaped { subject });
            },
            Instruction::Raw(subject) => if let Some(subject) = generate_expression(
                &mut subject.into_iter().peekable(),
                base.clone(),
                suffix
            ) {
                output.push(Control::Raw { subject });
            } else {
                output.push(Control::Text {
                    value: inst_pos.get_source(source),
                });
            },
            Instruction::IfStart(subject) => {
                if let Some(subject) = generate_expression(
                    &mut subject.into_iter().peekable(),
                    base.clone(),
                    suffix
                ) {
                    let (body, last) = parse_tree(source, input.by_ref(), base.clone(), suffix);

                    let alt = match last {
                        Some(InstructionPos { inst: Instruction::Else, .. }) => {
                            let (a, _) = parse_tree(source, input.by_ref(), base.clone(), suffix);
                            a
                        },
                        _ => Vec::new(),
                    };

                    output.push(Control::If {
                        subject,
                        body,
                        alt,
                    });
                } else {
                    output.push(Control::Text {
                        value: inst_pos.get_source(source),
                    });
                }
            },
            Instruction::IterStart(subject) => {
                if let Some(subject) = generate_expression(
                    &mut subject.into_iter().peekable(),
                    base.clone(),
                    suffix
                ) {
                    let path = match &subject {
                        Expression::PathExpression { path } => path.clone(),
                        _ => Vec::new(),
                    };

                    let (body, last) = parse_tree(source, input.by_ref(), paths::iter_element(path.clone(), suffix), suffix + 1);

                    let alt = match last {
                        Some(InstructionPos { inst: Instruction::Else, .. }) => {
                            let (a, _) = parse_tree(source, input.by_ref(), paths::iter_element(path.clone(), suffix), suffix + 1);
                            a
                        },
                        _ => Vec::new(),
                    };

                    output.push(Control::Iter {
                        suffix,
                        subject_raw: path.join("."),
                        subject,
                        body,
                        alt,
                    });
                } else {
                    output.push(Control::Text {
                        value: inst_pos.get_source(source),
                    });
                }
            },
            Instruction::Else | Instruction::End(_) => {
                last = Some(inst_pos);
                break;
            },
        }
    }

    (output, last)
}
