use token::{ Token, TokenPos };

/// iterate a slice over a string
/// the slice can be varying sizes and vary in position
#[derive(Debug, Clone)]
struct StringSlicer<'a> {
    source: &'a str,
    start: usize,
    end: usize,
}

impl<'a> StringSlicer<'a> {
    fn new(input: &'a str) -> StringSlicer<'a> {
        StringSlicer {
            source: input,
            start: 0,
            end: 1,
        }
    }

    /// get current slice
    fn slice(&self) -> Option<String> {
        self.source.get(self.start..self.end).map(|x| x.to_string())
    }

    /// reset slice to length of 1
    fn reset(&mut self) {
        self.end = self.start + 1;
    }

    /// move the beginning right one, reset length to 1
    fn step(&mut self) {
        self.start = self.end;
        self.reset();
    }

    /// step by `inc` units
    fn step_by(&mut self, inc: usize) {
        self.start = self.end + inc - 1;
        self.reset();
    }
    
    /// increment right end of slice, keeping the beginning in place
    fn grow(&mut self) {
        self.end += 1;
    }

    /// grow by `inc` units
    fn grow_by(&mut self, inc: usize) {
        self.end += inc;
    }

    /// get substr preceeding slice
    // fn prefix(&self, length: usize) -> Option<&str> {
    //     if self.start < length {
    //         None
    //     } else {
    //         self.source.get((self.start - length)..self.start)
    //     }
    // }

    /// see character directly following slice
    fn suffix(&self, length: usize) -> Option<String> {
        self.source.get(self.end..(self.end + length)).map(|x| x.to_string())
    }

    /// check if slice is followed by the given string
    fn followed_by(&self, target: &str) -> bool {
        if let Some(substr) = self.source.get(self.end..(self.end + target.len())) {
            substr == target
        } else { false }
    }

    /// step until current slice is not a single space
    fn skip_spaces(&mut self) {
        while self.slice() == Some(" ".to_string()) {
            self.step();
        }
    }
}

fn is_simple_char(ch: char) -> bool {
    ch.is_alphabetic() || ch.is_numeric() || match ch {
        '@' | '/' | '_' | ':' | '\\' | '-' | '.' => true,
        _ => false,
    }
}

/// lex an expression from the current slice position
/// return an option of the token vector representing the expression
fn lex_expression(slicer: &mut StringSlicer) -> Option<Vec<TokenPos>> {
    let mut output: Vec<TokenPos> = Vec::new();

    slicer.skip_spaces();

    if let Some(slice) = slicer.slice() {
        match slice.as_ref() {
            // string literals
            "\"" => {
                let start = slicer.start;
                slicer.step();

                while let Some(mut string_lit) = slicer.slice() {
                    match string_lit.chars().last() {
                        // grow to include backslash and escaped char
                        Some('\\') => slicer.grow_by(2),
                        // finish the string
                        Some('"') => {
                            // skip last character
                            string_lit.pop();

                            slicer.step();
                            return Some(vec![
                                TokenPos {
                                    start,
                                    end: slicer.start,
                                    tok: Token::StringLiteral(string_lit),
                                }
                            ]);
                        },
                        Some(_) => slicer.grow(),
                        None => return None,
                    }
                }
            },
            // if not ...
            "!" => {
                output.push(TokenPos {
                    start: slicer.start,
                    end: slicer.end,
                    tok: Token::Bang,
                });
                slicer.step();

                if let Some(mut sub_expr) = lex_expression(slicer) {
                    output.append(&mut sub_expr);
                } else { return None; }
            },
            // identifier or helper
            _ => if slice.chars().all(|ch| ch != '-' && is_simple_char(ch)) {
                // collect simple chars for identifier
                while let Some(_) = slicer.slice() {
                    if let Some(suffix) = slicer.suffix(1) {
                        if is_simple_char(suffix.chars().nth(0).unwrap()) {
                            slicer.grow();
                        } else { break; }
                    } else { break; }
                }

                if let Some(sub_slice) = slicer.slice() {
                    // legacy helper call
                    if sub_slice.starts_with("function.") {
                        let helper_name = sub_slice[9..].to_string();
                        output.push(TokenPos {
                            start: slicer.start,
                            end: slicer.start + 9,
                            tok: Token::LegacyHelper,
                        });
                        output.push(TokenPos {
                            start: slicer.start + 9,
                            end: slicer.end,
                            tok: Token::Identifier(helper_name),
                        });

                        slicer.step();
                        slicer.skip_spaces();

                        // get arguments
                        while slicer.slice() == Some(",".to_string()) {
                            output.push(TokenPos {
                                start: slicer.start,
                                end: slicer.end,
                                tok: Token::Comma,
                            });
                            slicer.step();
                            
                            if let Some(mut arg) = lex_expression(slicer) {
                                output.append(&mut arg);
                            }
                            // allow a trailing comma

                            slicer.skip_spaces();
                        }
                    } else {
                        let name = sub_slice.to_string();
                        output.push(TokenPos {
                            start: slicer.start,
                            end: slicer.end,
                            tok: Token::Identifier(name),
                        });

                        slicer.step();
                        slicer.skip_spaces();

                        // helper call
                        if slicer.slice() == Some("(".to_string()) {
                            output.push(TokenPos {
                                start: slicer.start,
                                end: slicer.end,
                                tok: Token::LeftParen,
                            });

                            // get arguments
                            while {
                                slicer.step();

                                if let Some(mut arg) = lex_expression(slicer) {
                                    output.append(&mut arg);
                                }
                                // allow a trailing comma

                                slicer.skip_spaces();

                                if slicer.slice() == Some(",".to_string()) {
                                    output.push(TokenPos {
                                        start: slicer.start,
                                        end: slicer.end,
                                        tok: Token::Comma,
                                    });
                                    true
                                } else { false }
                            } {}

                            if slicer.slice() == Some(")".to_string()) {
                                output.push(TokenPos {
                                    start: slicer.start,
                                    end: slicer.end,
                                    tok: Token::RightParen,
                                });
                                slicer.step();
                            } else { return None; }
                        }
                    }
                } else { return None; }
            } else { return None; },
        }
    }

    slicer.skip_spaces();

    Some(output)
}

/// lex a block (`if expr`, `each expr`, `else`, `end`)
fn lex_block(slicer: &mut StringSlicer) -> Option<Vec<TokenPos>> {
    let mut output: Vec<TokenPos> = Vec::new();

    slicer.skip_spaces();
    slicer.grow_by(2);

    if let Some(slice) = slicer.slice() {
        match slice.as_ref() {
            // if tokens
            "if " | "IF " => {
                output.push(TokenPos {
                    start: slicer.start,
                    end: slicer.end - 1,
                    tok: Token::If,
                });
                slicer.step();
                slicer.skip_spaces();

                if let Some(mut expr) = lex_expression(slicer) {
                    output.append(&mut expr);
                } else { return None; }
            },
            // iterator tokens
            "eac" | "BEG" => if match slice.as_ref() {
                "eac" => if slicer.followed_by("h ") {
                    slicer.grow();
                    true
                } else { false },
                // "BEG"
                _ => if slicer.followed_by("IN ") {
                    slicer.grow_by(2);
                    true
                } else { false },
            } {
                output.push(TokenPos {
                    start: slicer.start,
                    end: slicer.end,
                    tok: Token::Iter,
                });
                slicer.step();
                slicer.skip_spaces();

                if let Some(mut expr) = lex_expression(slicer) {
                    output.append(&mut expr);
                } else { return None; }
            } else { return None; },
            // end tokens
            "end" | "END" => {
                if slice == "END" && slicer.followed_by("IF") {
                    slicer.grow_by(2);
                }

                output.push(TokenPos {
                    start: slicer.start,
                    end: slicer.end,
                    tok: Token::End,
                });
                slicer.step();
                slicer.skip_spaces();

                if let Some(mut expr) = lex_expression(slicer) {
                    output.append(&mut expr);
                }
                // end subject is optional
            },
            // else tokens
            "els" | "ELS" => if match slice.as_ref() {
                "els" => slicer.followed_by("e"),
                // "ELS"
                _ => slicer.followed_by("E"),
            } {
                slicer.grow();
                output.push(TokenPos {
                    start: slicer.start,
                    end: slicer.end,
                    tok: Token::Else,
                });
                slicer.step();
            } else { return None; },
            _ => { return None; }
        }
    }

    slicer.skip_spaces();

    Some(output)
}

/// lex the input string into Tokens
pub fn lex(input: &str) -> Vec<TokenPos> {
    let mut output: Vec<TokenPos> = vec![];
    let length = input.len();
    let mut slicer = StringSlicer::new(input);

    while let Some(slice) = slicer.slice() {
        match slice.as_ref() {
            // escaped opens
            "\\" => {
                if let Some(target) = ["<!--", "{{{", "{{", "{"].iter().find(|x| slicer.followed_by(x)) {
                    let len = target.len();
                    
                    slicer.step();
                    output.push(TokenPos {
                        start: slicer.start,
                        end: slicer.start + len,
                        tok: Token::Text(target.to_string()),
                    });
                    slicer.step_by(len);
                } else {
                    slicer.grow();
                }
            },
            // escaped or raw mustache
            "{" | "{{" => if slicer.followed_by("{") {
                slicer.grow();
            } else {
                let start = slicer.start;
                let orig_end = slicer.end;

                let mut copy = slicer.clone();
                copy.step();

                let valid = if let Some(mut tokens) = lex_expression(&mut copy) {
                    let closer = match slice.as_ref() {
                        "{" => "}",
                        // "{{"
                        _ => {
                            copy.grow();
                            "}}"
                        },
                    }.to_string();

                    if copy.slice() == Some(closer) {
                        let (open_token, close_token) = match slice.as_ref() {
                            "{" => (TokenPos {
                                start,
                                end: orig_end,
                                tok: Token::EscapedOpen,
                            }, TokenPos {
                                start: copy.start,
                                end: copy.end,
                                tok: Token::EscapedClose,
                            }),
                            // "{{"
                            _ => (TokenPos {
                                start,
                                end: orig_end,
                                tok: Token::RawOpen,
                            }, TokenPos {
                                start: copy.start,
                                end: copy.end,
                                tok: Token::RawClose,
                            }),
                        };

                        output.push(open_token);
                        output.append(&mut tokens);
                        output.push(close_token);

                        slicer.step_by(copy.end - orig_end + 1);

                        true
                    } else { false }
                } else { false };

                if !valid {
                    output.push(TokenPos {
                        start: slicer.start,
                        end: slicer.end,
                        tok: Token::Text(slice)
                    });
                    slicer.step();
                }
            },
            // modern or legacy block
            "<!--" | "{{{" => {
                let start = slicer.start;
                let orig_end = slicer.end;

                let mut copy = slicer.clone();
                copy.step();

                let valid = if let Some(mut tokens) = lex_block(&mut copy) {
                    let closer_len = 3;
                    
                    let closer = match slice.as_ref() {
                        "<!--" => "-->",
                        // "{{{"
                        _ => "}}}",
                    }.to_string();

                    copy.grow_by(2);

                    if copy.slice() == Some(closer) {
                        output.push(TokenPos {
                            start,
                            end: orig_end,
                            tok: Token::BlockOpen,
                        });
                        output.append(&mut tokens);
                        output.push(TokenPos {
                            start: copy.start,
                            end: copy.end,
                            tok: Token::BlockClose,
                        });

                        slicer.step_by(copy.end + closer_len - orig_end - 2);

                        true
                    } else { false }
                } else { false };

                if !valid {
                    output.push(TokenPos {
                        start: slicer.start,
                        end: slicer.end,
                        tok: Token::Text(slice)
                    });
                    slicer.step();
                }
            },
            // text
            _ => {
                // start of an instruction
                if slicer.followed_by("\\") || slicer.followed_by("{") || slicer.followed_by("<!--") {
                    output.push(TokenPos {
                        start: slicer.start,
                        end: slicer.end,
                        tok: Token::Text(slice)
                    });
                    slicer.step();
                } else {
                    slicer.grow();
                }
            }
        }

        // add the last piece of text
        if slicer.end >= length {
            slicer.end = length;
            if let Some(text) = slicer.slice() {
                output.push(TokenPos {
                    start: slicer.start,
                    end: slicer.end,
                    tok: Token::Text(text)
                });
            }
            break;
        }
    }

    // collapse subsequent Text tokens
    let mut collapsed = vec![];
    let mut iter = output.into_iter();

    if let Some(mut prev) = iter.next() {
        for current in iter {
            match (prev.clone(), current) {
                (TokenPos { start, tok: Token::Text(a), .. }, TokenPos { end, tok: Token::Text(b), .. }) => {
                    prev = TokenPos {
                        start,
                        end,
                        tok: Token::Text(format!("{}{}", a, b)),
                    };
                },
                (copy, current) => {
                    if if let TokenPos { tok: Token::Text(val), .. } = copy {
                        val.len() > 0
                    } else { true } {
                        collapsed.push(prev);
                    }
                    prev = current;
                },
            }
        }

        if if let TokenPos { tok: Token::Text(val), .. } = &prev {
            val.len() > 0
        } else { true } {
            collapsed.push(prev);
        }
    }

    collapsed
}

#[cfg(test)]
mod tests {
    use super::*;

    fn to_tokens(from: Option<Vec<TokenPos>>) -> Vec<Token> {
        from.unwrap().into_iter().map(|TokenPos { tok, .. }| tok).collect()
    }

    // lex expression tests
    #[test]
    fn string_lit() {
        assert_eq!(
            to_tokens(lex_expression(&mut StringSlicer::new("\"\\\\ \\ \""))),
            vec![Token::StringLiteral(r"\\ \ ".to_string())]
        );

        assert_eq!(
            to_tokens(lex_expression(&mut StringSlicer::new("\"help me to save myself\""))),
            vec![Token::StringLiteral("help me to save myself".to_string())]
        );

        assert_eq!(
            to_tokens(lex_expression(&mut StringSlicer::new("function.caps, \"help me to save myself\""))),
            vec![
                Token::LegacyHelper,
                Token::Identifier("caps".to_string()),
                Token::Comma,
                Token::StringLiteral("help me to save myself".to_string())
            ]
        );
    }

    #[test]
    fn bang() {
        assert_eq!(
            to_tokens(lex_expression(&mut StringSlicer::new("!name_of"))),
            vec![
                Token::Bang,
                Token::Identifier("name_of".to_string()),
            ]
        );

        assert_eq!(
            to_tokens(lex_expression(&mut StringSlicer::new("!name_of extra stuff"))),
            vec![
                Token::Bang,
                Token::Identifier("name_of".to_string()),
            ]
        );

        assert_eq!(
            to_tokens(lex_expression(&mut StringSlicer::new(" ! rooms.private"))),
            vec![
                Token::Bang,
                Token::Identifier("rooms.private".to_string()),
            ]
        );
    }

    #[test]
    fn identifier() {
        assert_eq!(
            to_tokens(lex_expression(&mut StringSlicer::new("name_of"))),
            vec![Token::Identifier("name_of".to_string())]
        );
    }

    #[test]
    fn legacy_helper() {
        assert_eq!(
            to_tokens(lex_expression(&mut StringSlicer::new("function.helper_name , arg1, arg2"))),
            vec![
                Token::LegacyHelper,
                Token::Identifier("helper_name".to_string()),
                Token::Comma,
                Token::Identifier("arg1".to_string()),
                Token::Comma,
                Token::Identifier("arg2".to_string()),
            ]
        );
    }

    #[test]
    fn modern_helper() {
        assert_eq!(
            to_tokens(lex_expression(&mut StringSlicer::new("helper_name(arg1, arg2 , )"))),
            vec![
                Token::Identifier("helper_name".to_string()),
                Token::LeftParen,
                Token::Identifier("arg1".to_string()),
                Token::Comma,
                Token::Identifier("arg2".to_string()),
                Token::Comma,
                Token::RightParen,
            ]
        );

        assert_eq!(
            to_tokens(lex_expression(&mut StringSlicer::new("helper_name() after stuff"))),
            vec![
                Token::Identifier("helper_name".to_string()),
                Token::LeftParen,
                Token::RightParen,
            ]
        );
    }

    // lex block tests
    #[test]
    fn if_block() {
        assert_eq!(
            to_tokens(lex_block(&mut StringSlicer::new("if abc"))),
            vec![
                Token::If,
                Token::Identifier("abc".to_string()),
            ]
        );

        assert_eq!(
            to_tokens(lex_block(&mut StringSlicer::new("IF foo.bar"))),
            vec![
                Token::If,
                Token::Identifier("foo.bar".to_string()),
            ]
        );

        assert_eq!(
            to_tokens(lex_block(&mut StringSlicer::new("if !valid(stuff)"))),
            vec![
                Token::If,
                Token::Bang,
                Token::Identifier("valid".to_string()),
                Token::LeftParen,
                Token::Identifier("stuff".to_string()),
                Token::RightParen,
            ]
        );

        assert_eq!(
            to_tokens(lex_block(&mut StringSlicer::new("IF foo.bar extra stuff"))),
            vec![
                Token::If,
                Token::Identifier("foo.bar".to_string()),
            ]
        );
    }

    #[test]
    fn iter_block() {
        assert_eq!(
            to_tokens(lex_block(&mut StringSlicer::new("each abc"))),
            vec![
                Token::Iter,
                Token::Identifier("abc".to_string()),
            ]
        );

        assert_eq!(
            to_tokens(lex_block(&mut StringSlicer::new("BEGIN foo.bar"))),
            vec![
                Token::Iter,
                Token::Identifier("foo.bar".to_string()),
            ]
        );

        assert_eq!(
            to_tokens(lex_block(&mut StringSlicer::new("each valid(stuff)"))),
            vec![
                Token::Iter,
                Token::Identifier("valid".to_string()),
                Token::LeftParen,
                Token::Identifier("stuff".to_string()),
                Token::RightParen,
            ]
        );

        assert_eq!(
            to_tokens(lex_block(&mut StringSlicer::new("BEGIN foo.bar extra stuff"))),
            vec![
                Token::Iter,
                Token::Identifier("foo.bar".to_string()),
            ]
        );
    }

    #[test]
    fn end_block() {
        assert_eq!(
            to_tokens(lex_block(&mut StringSlicer::new("end abc"))),
            vec![
                Token::End,
                Token::Identifier("abc".to_string()),
            ]
        );

        assert_eq!(
            to_tokens(lex_block(&mut StringSlicer::new("ENDIF foo.bar"))),
            vec![
                Token::End,
                Token::Identifier("foo.bar".to_string()),
            ]
        );

        assert_eq!(
            to_tokens(lex_block(&mut StringSlicer::new("END valid(stuff)"))),
            vec![
                Token::End,
                Token::Identifier("valid".to_string()),
                Token::LeftParen,
                Token::Identifier("stuff".to_string()),
                Token::RightParen,
            ]
        );

        assert_eq!(
            to_tokens(lex_block(&mut StringSlicer::new("ENDIF foo.bar extra stuff"))),
            vec![
                Token::End,
                Token::Identifier("foo.bar".to_string()),
            ]
        );
    }
}
