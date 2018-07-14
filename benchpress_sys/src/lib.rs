#[macro_use]
extern crate lazy_static;
extern crate regex;
extern crate onig;
extern crate itertools;
extern crate json;

pub mod token;
pub mod lexer;
pub mod instruction;
pub mod parser;
pub mod paths;
pub mod pre_fixer;
pub mod templates;
pub mod generator;

pub fn compile(template: String) -> String {
    let pre_fixed = pre_fixer::pre_fix(template);
    let lexed = lexer::lex(&pre_fixed);
    let first_parsed = parser::parse_instructions(&pre_fixed, lexed);
    let extras_fixed = parser::fix_extra_tokens(&pre_fixed, first_parsed);
    let (tree, _) = parser::parse_tree(&pre_fixed, &mut extras_fixed.into_iter(), Vec::new(), 1);
    let code = generator::generate(tree);

    code
}
