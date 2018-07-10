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
    let first_lexed = lexer::first_pass(pre_fixed.clone());
    let second_lexed = lexer::second_pass(first_lexed.clone());
    let first_parsed = parser::first_pass(second_lexed.clone());
    let extras_fixed = parser::fix_extra_tokens(first_parsed.clone());
    let (tree, _) = parser::second_pass(&mut extras_fixed.clone().into_iter().peekable(), Vec::new(), 1);

    let code = generator::generate(tree.clone());

    code
}
