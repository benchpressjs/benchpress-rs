extern crate benchpress_sys;

use benchpress_sys::{parser, pre_fixer, lexer, generator};
use parser::Control;

use std::io::{self, BufRead, Write, Read};

fn tree_tostring(tree: Vec<Control>) -> String {
    let mut output = String::new();
    
    for elem in tree {
        output.push_str(match elem {
            Control::If { subject, body, alt } => format!(
                "If {{ subject: {:?}, body: {}, alt: {} }},",
                subject, tree_tostring(body), tree_tostring(alt)
            ),
            Control::Iter { subject_raw, suffix, subject, body, alt } => format!(
                "Iter {{ suffix: {}, raw: {}, subject: {:?}, body: {}, alt: {} }},",
                suffix, subject_raw, subject, tree_tostring(body), tree_tostring(alt)
            ),
            _ => format!("{:?},", elem),
        }.as_ref());

        output.push('\n');
    }

    output = output.lines().map(|x| format!("  {}", x)).collect::<Vec<String>>().join("\n");

    if output.len() > 0 {
        format!("[\n{}\n]", output)
    } else {
        String::from("[]")
    }
}

fn go(input: String, debug: bool) {
    let pre_fixed = pre_fixer::pre_fix(input);
    let lexed = lexer::lex(&pre_fixed);
    let first_parsed = parser::parse_instructions(&pre_fixed, lexed.clone());
    let extras_fixed = parser::fix_extra_tokens(&pre_fixed, first_parsed.clone());
    let (tree, _) = parser::parse_tree(&pre_fixed, &mut extras_fixed.clone().into_iter(), Vec::new(), 1);

    let code = generator::generate(tree.clone());

    if debug {
        println!("/*");

        println!("pre fixed   \n-------------\n{}\n\n", pre_fixed);
        let lexed_fixed = format!("{:?}", lexed);
        println!("lexed       \n-------------\n{}\n\n", lexed_fixed);
        let first_parsed_fixed = format!("{:?}", first_parsed);
        println!("first parsed\n-------------\n{}\n\n", first_parsed_fixed);
        let extras_fixed_fixed = format!("{:?}", extras_fixed);
        println!("extras fixed\n-------------\n{}\n\n", extras_fixed_fixed);
        println!("parse tree  \n-------------\n{}\n\n", tree_tostring(tree));

        println!("code        \n-------------*/");
    }

    println!("{}", code);
}

fn main() {
    // println!("Hello, world!");

    let stdin = io::stdin();

    let mut args = std::env::args();
    let debug = std::env::args().any(|x| x == "--debug");

    if args.any(|x| x == "-") {
        let mut passed = String::new();
        stdin.lock().read_to_string(&mut passed).expect("Failed to read stdin");

        go(passed, debug);
    } else {
        loop {
            // Stdout needs to be flushed, due to missing newline
            print!(">> ");
            io::stdout().flush().expect("Error flushing stdout");

            let mut line = String::new();
            stdin.lock().read_line(&mut line).expect("Error reading from stdin");
            
            go(line, debug);
        }
    }
}
