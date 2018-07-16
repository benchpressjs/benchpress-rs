use token::{Token, TokenPos};
use instruction::{Instruction, InstructionPos};

use std::iter::{Iterator, Peekable};
use itertools::Itertools;

/// parse the lexer output (like `{{each people}}` and `{stuff}`)
/// into instructions
pub fn parse_instructions(_source: &str, tokens: Vec<TokenPos>) -> Vec<InstructionPos> {
    let mut output: Vec<InstructionPos> = vec![];

    let mut iter = tokens.into_iter().peekable();

    while let Some(opener) = iter.next() {
        if let Some(inst_pos) = match opener {
            // convert a text token to a text instruction
            TokenPos { tok: Token::Text(text), start, end } => Some(InstructionPos {
                start,
                end,
                inst: Instruction::Text(text),
            }),
            // parse blocks into instructions
            TokenPos { tok: Token::BlockOpen, start, .. } => {
                let TokenPos { tok: keyword, .. } = iter.next().unwrap();

                // collect tokens for expression
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
            // parse interpolation mustaches into instructions
            TokenPos { tok: Token::RawOpen, start, .. } | 
            TokenPos { tok: Token::EscapedOpen, start, .. } => {
                let closer = match opener {
                    TokenPos { tok: Token::RawOpen, .. } => Token::RawClose,
                    // TokenPos { tok: Token::EscapedOpen, .. }
                    _ => Token::EscapedClose,
                };

                // collect tokens for expression
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

/// check if a vector starts with the elements of another vector
pub fn starts_with<T>(full: &Vec<T>, part: &Vec<T>) -> bool
where
    T: Eq
{
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

/// in a case where there are extra End instructions
/// try to match them to Ifs or Iters
/// and remove the extra ones
pub fn fix_extra_instructions(source: &str, input: Vec<InstructionPos>) -> Vec<InstructionPos> {
    let mut remove: HashSet<InstructionPos> = HashSet::new();
    let mut expected_subjects: Vec<Vec<Token>> = Vec::new();

    let mut starts_count: u16 = 0;
    let mut ends_count: u16 = 0;

    // try to find a Close with no corresponding Open
    for index in 0..input.len() {
        let elem = &input[index];

        match elem {
            // for an Open, add the subject to the stack of expected subjects
            InstructionPos { inst: Instruction::IfStart(subject), .. } | 
            InstructionPos { inst: Instruction::IterStart(subject), .. } => {
                expected_subjects.push(subject.clone());
                starts_count += 1;
            },
            InstructionPos { inst: Instruction::End(subject), .. } => {
                ends_count += 1;

                if let Some(expected_subject) = expected_subjects.pop() {
                    if subject.len() > 0 && !starts_with(&expected_subject, subject) {
                        // doesn't start with what we expect, so remove it
                        remove.insert(elem.clone());
                        expected_subjects.push(expected_subject);
                    } else {
                        // search for an end within close proximity
                        // that has the expected subject
                        for i in (index + 1)..input.len() {
                            match &input[i] {
                                InstructionPos { inst: Instruction::IfStart(_), .. } |
                                InstructionPos { inst: Instruction::IterStart(_), .. } => {
                                    break;
                                },
                                InstructionPos { inst: Instruction::End(ahead_subject), .. } => {
                                    if ahead_subject == &expected_subject {
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
                    // no subject expected, so remove it
                    remove.insert(elem.clone());
                }
            },
            _ => (),
        }
    }

    // remove the number of instructions that are extra
    if ends_count > starts_count {
        let mut diff = ends_count - starts_count;

        println!("Found extra token(s):");

        let output: Vec<InstructionPos> = input.into_iter().map(|inst| if remove.contains(&inst) && diff > 0 {
            let inst_source = inst.get_source(source);
            println!("{}", inst_source);

            diff -= 1;
            // replace removed instructions with their source Text
            InstructionPos {
                start: inst.start,
                end: inst.end,
                inst: Instruction::Text(inst_source),
            }
        } else { inst }).collect();

        println!("These tokens will be passed through as text, but you should remove them to prevent issues in the future.");

        output
    } else {
        input
    }
}

/// an expression specified inside an instruction
#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    HelperExpression { helper_name: String, args: Vec<Expression> },
    PathExpression { path: Vec<String> },
    StringLiteral { value: String },
    NegativeExpression { expr: Box<Expression> },
}

/// built from instructions
/// controls how the template behaves
#[derive(Debug, PartialEq, Clone)]
pub enum Control {
    Text { value: String },
    If { subject: Expression, body: Vec<Control>, alt: Vec<Control> },
    Iter { suffix: u16, subject_raw: String, subject: Expression, body: Vec<Control>, alt: Vec<Control> },
    Escaped { subject: Expression },
    Raw { subject: Expression },
}

use paths;

/// generate an expression from an interator of Tokens
fn generate_expression<I>(iter: &mut Peekable<I>, base: &Vec<String>, suffix: u16) -> Option<Expression>
where
    I: Iterator<Item = Token>
{
    let first = iter.next();
    let second = iter.peek().map(|x| x.clone());

    match (first, second) {
        // negative expression (`!stuff`)
        (Some(Token::Bang), Some(_)) => if let Some(expr) = generate_expression(iter.by_ref(), base, suffix) {
            Some(Expression::NegativeExpression {
                expr: Box::new(expr),
            })
        } else {
            None
        },
        // helper expression (`function.name, arg1, arg2`, `name(arg1, arg2)`)
        (Some(Token::Identifier(name)), Some(Token::LeftParen)) |
        (Some(Token::LegacyHelper), Some(Token::Identifier(name))) => {
            let end = match iter.next() {
                Some(Token::LeftParen) => if iter.peek() == Some(&Token::RightParen) {
                    return Some(Expression::HelperExpression {
                        helper_name: name.to_string(),
                        args: Vec::new(),
                    })
                } else {
                    Some(Token::RightParen)
                },
                _ => {
                    // skip first comma
                    iter.next();
                    None
                },
            };
            let mut args: Vec<Expression> = Vec::new();

            // get arguments
            while {
                if let Some(arg) = generate_expression(iter.by_ref(), base, suffix) {
                    args.push(arg);
                }

                match &iter.next() {
                    Some(Token::Comma) => true,
                    x if x == &end => false,
                    _ => return None,
                }
            } {}

            Some(Expression::HelperExpression {
                helper_name: name.to_string(),
                args,
            })
        },
        // string literal (`"a literal string"`)
        (Some(Token::StringLiteral(value)), _) => Some(Expression::StringLiteral {
            value
        }),
        // identifier (`object.prop`, `../name`)
        (Some(Token::Identifier(value)), _) => {
            let path: Vec<String> = paths::split(value);

            Some(Expression::PathExpression { path: paths::resolve(&base, path) })
        },
        _ => None,
    }
}

/// build the tree
pub fn parse_tree<I>(
    source: &str,
    input: &mut I,
    base: &Vec<String>,
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
            // convert a text instruction to a text control
            Instruction::Text(value) => output.push(Control::Text { value }),
            // convert instruction to control
            // generate expression
            Instruction::Escaped(subject) => if let Some(subject) = generate_expression(
                &mut subject.into_iter().peekable(),
                base,
                suffix
            ) {
                output.push(Control::Escaped { subject });
            },
            Instruction::Raw(subject) => if let Some(subject) = generate_expression(
                &mut subject.into_iter().peekable(),
                base,
                suffix
            ) {
                output.push(Control::Raw { subject });
            },
            // create an if-then-else control
            Instruction::IfStart(subject) => if let Some(subject) = generate_expression(
                &mut subject.into_iter().peekable(),
                base,
                suffix
            ) {
                // recursively parse for body and alt child trees
                let (body, last) = parse_tree(source, input.by_ref(), base, suffix);

                let alt = match last {
                    Some(InstructionPos { inst: Instruction::Else, .. }) => {
                        let (a, _) = parse_tree(source, input.by_ref(), base, suffix);
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
            },
            // create an iteration control
            Instruction::IterStart(subject) => if let Some(subject) = generate_expression(
                &mut subject.into_iter().peekable(),
                base,
                suffix
            ) {
                // use base if there's not a path
                let path = match &subject {
                    Expression::PathExpression { path } => path.clone(),
                    _ => base.clone(),
                };

                // recursively parse for body and alt child trees
                let (body, last) = parse_tree(source, input.by_ref(), &paths::iter_element(&path, suffix), suffix + 1);

                let alt = match last {
                    Some(InstructionPos { inst: Instruction::Else, .. }) => {
                        let (a, _) = parse_tree(source, input.by_ref(), &paths::iter_element(&path, suffix), suffix + 1);
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
            },
            Instruction::Else | Instruction::End(_) => {
                last = Some(inst_pos);
                break;
            },
        }
    }

    (output, last)
}
