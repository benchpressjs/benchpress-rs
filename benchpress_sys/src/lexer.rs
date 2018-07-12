use token::Token;

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

    /// step until current slice does not contain a space
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

fn lex_expression(slicer: &mut StringSlicer) -> Option<Vec<Token>> {
    let mut output: Vec<Token> = Vec::new();

    slicer.skip_spaces();

    if let Some(slice) = slicer.slice() {
        match slice.as_ref() {
            // string literals
            "\"" => {
                slicer.step();

                while let Some(string_lit) = slicer.slice() {
                    match slicer.suffix(1).map(|x| x.chars().nth(0).unwrap()) {
                        // grow to include backslash and escaped char
                        Some('\\') => slicer.grow_by(2),
                        // finish the string
                        Some('"') => {
                            output.push(Token::StringLiteral(string_lit.to_string()));
                            slicer.step();
                            break;
                        },
                        Some(_) => slicer.grow(),
                        None => { return None; },
                    }
                }
            },
            // if not ...
            "!" => {
                output.push(Token::Bang);
                slicer.step();

                if let Some(mut sub_expr) = lex_expression(slicer) {
                    output.append(&mut sub_expr);
                } else { return None; }
            },
            // identifier or helper
            _ => {
                if slice.chars().all(|ch| is_simple_char(ch)) {
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
                            output.push(Token::LegacyHelper);
                            output.push(Token::Identifier(helper_name));

                            slicer.step();
                            slicer.skip_spaces();

                            while slicer.slice() == Some(",".to_string()) {
                                output.push(Token::Comma);
                                slicer.step();
                                
                                if let Some(mut arg) = lex_expression(slicer) {
                                    output.append(&mut arg);
                                }
                                // allow a trailing comma

                                slicer.skip_spaces();
                            }
                        } else {
                            let name = sub_slice.to_string();
                            output.push(Token::Identifier(name));

                            slicer.step();
                            slicer.skip_spaces();

                            if slicer.slice() == Some("(".to_string()) {
                                output.push(Token::LeftParen);

                                while {
                                    slicer.step();

                                    if let Some(mut arg) = lex_expression(slicer) {
                                        output.append(&mut arg);
                                    }
                                    // allow a trailing comma

                                    slicer.skip_spaces();

                                    if slicer.slice() == Some(",".to_string()) {
                                        output.push(Token::Comma);
                                        true
                                    } else { false }
                                } {}

                                if slicer.slice() == Some(")".to_string()) {
                                    output.push(Token::RightParen);
                                    slicer.step();
                                } else { return None; }
                            }
                        }
                    } else { return None; }
                } else { return None; }
            },
        }
    }

    slicer.skip_spaces();

    Some(output)
}

fn lex_block(slicer: &mut StringSlicer) -> Option<Vec<Token>> {
    let mut output: Vec<Token> = Vec::new();

    slicer.skip_spaces();
    slicer.grow_by(2);

    if let Some(slice) = slicer.slice() {
        match slice.as_ref() {
            "if " | "IF " => {
                output.push(Token::If);
                slicer.step();
                slicer.skip_spaces();

                if let Some(mut expr) = lex_expression(slicer) {
                    output.append(&mut expr);
                } else { return None; }
            },
            "eac" | "BEG" => if match slice.as_ref() {
                "eac" => slicer.followed_by("h "),
                "BEG" => slicer.followed_by("IN "),
                _ => false,
            } {
                output.push(Token::Iter);
                // step 3 times
                // if "each ", skips to after the "h "
                // if "BEGIN ", skips to after the "IN"
                slicer.step_by(3);
                slicer.skip_spaces();

                if let Some(mut expr) = lex_expression(slicer) {
                    output.append(&mut expr);
                } else { return None; }
            },
            "end" | "END" => {
                let (yes, inc) = if slicer.followed_by(" ") {
                    // step to the space, then to the next
                    (true, 2)
                } else if slicer.followed_by("IF ") {
                    // step to the I, F, and space, then to the next
                    (true, 4)
                } else { (false, 0) };

                if yes {
                    output.push(Token::End);
                    slicer.step_by(inc);
                    slicer.skip_spaces();

                    if let Some(mut expr) = lex_expression(slicer) {
                        output.append(&mut expr);
                    }
                    // end subject is optional
                }
            },
            "els" | "ELS" => if match slice.as_ref() {
                "els" => slicer.followed_by("e"),
                "ELS" => slicer.followed_by("E"),
                _ => false,
            } {
                output.push(Token::Else);
                slicer.step();
            },
            _ => { return None; }
        }
    }

    slicer.skip_spaces();

    Some(output)
}

pub fn lex(input: &str) -> Vec<Token> {
    let mut output: Vec<Token> = vec![];
    let length = input.len();
    let mut slicer = StringSlicer::new(input);

    while let Some(slice) = slicer.slice() {
        match slice.as_ref() {
            // escaped opens
            "\\" => {
                if let Some(target) = ["<!--", "{{{", "{{", "{"].iter().find(|x| slicer.followed_by(x)) {
                    slicer.step();
                    output.push(Token::Text(target.to_string()));
                    slicer.step_by(target.len());
                }
            },
            // escaped or raw mustache
            "{" | "{{" => if slicer.followed_by("{") {
                slicer.grow();
            } else {
                let orig_end = slicer.end;

                let mut copy = slicer.clone();
                copy.step();

                println!("1 {:?}, {:?}, {:?}, {:?}", slicer, slicer.slice(), copy, copy.slice());

                let valid = if let Some(mut tokens) = lex_expression(&mut copy) {
                    let closer = match slice.as_ref() {
                        "{" => "}",
                        "{{" => {
                            copy.grow();
                            "}}"
                        },
                        _ => "",
                    }.to_string();

                    let closer_len = closer.len();

                    println!("2 {:?}, {:?}, {:?}, {:?}", slicer, slicer.slice(), copy, copy.slice());

                    if copy.slice() == Some(closer) {
                        let (open_token, close_token) = match slice.as_ref() {
                            "{" => (Token::EscapedOpen, Token::EscapedClose),
                            "{{" => (Token::RawOpen, Token::RawClose),
                            _ => (Token::RawOpen, Token::RawClose),
                        };

                        println!("3 {:?}, {:?}, {:?}, {:?}", slicer, slicer.slice(), copy, copy.slice());

                        output.push(open_token);
                        output.append(&mut tokens);
                        output.push(close_token);

                        slicer.step_by(copy.end + closer_len - orig_end);

                        true
                    } else { false }
                } else { false };

                println!("4 {:?}, {:?}, {:?}, {:?}", slicer, slicer.slice(), copy, copy.slice());

                if !valid {
                    output.push(Token::Text(slice));
                    slicer.step();
                }
            },
            // modern or legacy block
            "<!--" | "{{{" => {
                let orig_end = slicer.end;

                let mut copy = slicer.clone();
                copy.step();

                let valid = if let Some(mut tokens) = lex_block(&mut copy) {
                    let closer_len = 3;
                    
                    let closer = match slice.as_ref() {
                        "<!--" => "-->",
                        "{{{" => "}}}",
                        _ => "   ",
                    }.to_string();

                    copy.grow_by(2);

                    if copy.slice() == Some(closer) {
                        output.push(Token::BlockOpen);
                        output.append(&mut tokens);
                        output.push(Token::BlockClose);

                        slicer.step_by(copy.end + closer_len - orig_end);

                        true
                    } else { false }
                } else { false };

                if !valid {
                    output.push(Token::Text(slice));
                    slicer.step();
                }
            },
            // text
            _ => {
                if slicer.followed_by("{") || slicer.followed_by("<!--") {
                    output.push(Token::Text(slice));
                    slicer.step();
                } else {
                    slicer.grow();
                }
            }
        }

        if slicer.end >= length {
            slicer.end = length;
            if let Some(text) = slicer.slice() {
                output.push(Token::Text(text));
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
                (Token::Text(a), Token::Text(b)) => {
                    prev = Token::Text(format!("{}{}", a, b));
                },
                (copy, current) => {
                    if if let Token::Text(text) = copy {
                        text.len() > 0 
                    } else { true } {
                        collapsed.push(prev);
                    }
                    prev = current;
                },
            }
        }

        if if let Token::Text(text) = &prev {
            text.len() > 0 
        } else { true } {
            collapsed.push(prev);
        }
    }

    collapsed
}

#[cfg(test)]
mod tests {
    use super::*;

    // lex expression tests
    #[test]
    fn string_lit() {
        assert_eq!(
            lex_expression(&mut StringSlicer::new("\"\\\\ \\ \"")),
            Some(vec![Token::StringLiteral(r"\\ \ ".to_string())])
        );

        assert_eq!(
            lex_expression(&mut StringSlicer::new("\"\\\\ \\ \" }")),
            Some(vec![Token::StringLiteral(r"\\ \ ".to_string())])
        );
    }

    #[test]
    fn bang() {
        assert_eq!(
            lex_expression(&mut StringSlicer::new("!name_of")),
            Some(vec![
                Token::Bang,
                Token::Identifier("name_of".to_string()),
            ])
        );

        assert_eq!(
            lex_expression(&mut StringSlicer::new("!name_of extra stuff")),
            Some(vec![
                Token::Bang,
                Token::Identifier("name_of".to_string()),
            ])
        );

        assert_eq!(
            lex_expression(&mut StringSlicer::new(" ! rooms.private")),
            Some(vec![
                Token::Bang,
                Token::Identifier("rooms.private".to_string()),
            ])
        );
    }

    #[test]
    fn identifier() {
        assert_eq!(
            lex_expression(&mut StringSlicer::new("name_of")),
            Some(vec![Token::Identifier("name_of".to_string())])
        );
    }

    #[test]
    fn legacy_helper() {
        assert_eq!(
            lex_expression(&mut StringSlicer::new("function.helper_name , arg1, arg2")),
            Some(vec![
                Token::LegacyHelper,
                Token::Identifier("helper_name".to_string()),
                Token::Comma,
                Token::Identifier("arg1".to_string()),
                Token::Comma,
                Token::Identifier("arg2".to_string()),
            ])
        );
    }

    #[test]
    fn modern_helper() {
        assert_eq!(
            lex_expression(&mut StringSlicer::new("helper_name(arg1, arg2 , )")),
            Some(vec![
                Token::Identifier("helper_name".to_string()),
                Token::LeftParen,
                Token::Identifier("arg1".to_string()),
                Token::Comma,
                Token::Identifier("arg2".to_string()),
                Token::Comma,
                Token::RightParen,
            ])
        );

        assert_eq!(
            lex_expression(&mut StringSlicer::new("helper_name() after stuff")),
            Some(vec![
                Token::Identifier("helper_name".to_string()),
                Token::LeftParen,
                Token::RightParen,
            ])
        );
    }

    // lex block tests
    #[test]
    fn if_block() {
        assert_eq!(
            lex_block(&mut StringSlicer::new("if abc")),
            Some(vec![
                Token::If,
                Token::Identifier("abc".to_string()),
            ])
        );

        assert_eq!(
            lex_block(&mut StringSlicer::new("IF foo.bar")),
            Some(vec![
                Token::If,
                Token::Identifier("foo.bar".to_string()),
            ])
        );

        assert_eq!(
            lex_block(&mut StringSlicer::new("if !valid(stuff)")),
            Some(vec![
                Token::If,
                Token::Bang,
                Token::Identifier("valid".to_string()),
                Token::LeftParen,
                Token::Identifier("stuff".to_string()),
                Token::RightParen,
            ])
        );

        assert_eq!(
            lex_block(&mut StringSlicer::new("IF foo.bar extra stuff")),
            Some(vec![
                Token::If,
                Token::Identifier("foo.bar".to_string()),
            ])
        );
    }

    #[test]
    fn iter_block() {
        assert_eq!(
            lex_block(&mut StringSlicer::new("each abc")),
            Some(vec![
                Token::Iter,
                Token::Identifier("abc".to_string()),
            ])
        );

        assert_eq!(
            lex_block(&mut StringSlicer::new("BEGIN foo.bar")),
            Some(vec![
                Token::Iter,
                Token::Identifier("foo.bar".to_string()),
            ])
        );

        assert_eq!(
            lex_block(&mut StringSlicer::new("each valid(stuff)")),
            Some(vec![
                Token::Iter,
                Token::Identifier("valid".to_string()),
                Token::LeftParen,
                Token::Identifier("stuff".to_string()),
                Token::RightParen,
            ])
        );

        assert_eq!(
            lex_block(&mut StringSlicer::new("BEGIN foo.bar extra stuff")),
            Some(vec![
                Token::Iter,
                Token::Identifier("foo.bar".to_string()),
            ])
        );
    }

    #[test]
    fn end_block() {
        assert_eq!(
            lex_block(&mut StringSlicer::new("end abc")),
            Some(vec![
                Token::End,
                Token::Identifier("abc".to_string()),
            ])
        );

        assert_eq!(
            lex_block(&mut StringSlicer::new("ENDIF foo.bar")),
            Some(vec![
                Token::End,
                Token::Identifier("foo.bar".to_string()),
            ])
        );

        assert_eq!(
            lex_block(&mut StringSlicer::new("END valid(stuff)")),
            Some(vec![
                Token::End,
                Token::Identifier("valid".to_string()),
                Token::LeftParen,
                Token::Identifier("stuff".to_string()),
                Token::RightParen,
            ])
        );

        assert_eq!(
            lex_block(&mut StringSlicer::new("ENDIF foo.bar extra stuff")),
            Some(vec![
                Token::End,
                Token::Identifier("foo.bar".to_string()),
            ])
        );
    }
}
