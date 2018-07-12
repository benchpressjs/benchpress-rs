#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate onig;
extern crate itertools;
extern crate json;

pub mod lexer;
pub mod token;
pub mod parser;
pub mod paths;
pub mod pre_fixer;
pub mod templates;
pub mod generator;

pub fn compile(template: String) -> String {
    let pre_fixed = pre_fixer::pre_fix(template);
    let lexed = lexer::lex(pre_fixed.as_ref());
    let first_parsed = parser::first_pass(lexed);
    let extras_fixed = parser::fix_extra_tokens(first_parsed);
    let (tree, _) = parser::second_pass(&mut extras_fixed.into_iter().peekable(), Vec::new(), 1);
    let code = generator::generate(tree);

    code
}
