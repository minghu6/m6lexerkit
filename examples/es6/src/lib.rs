mod tokenizer1;
mod tokenizer2;

use m6tokenizer::Token;
use maplit::hashset;

pub use tokenizer1::tokenize as tokenize1;
pub use tokenizer2::tokenize as tokenize2;


pub(crate) fn trim_tokens(tokens: &[Token]) -> Vec<Token> {
    let blank_set = hashset! { "newline", "sp", "slash_line_comment" };

    tokens
    .iter()
    .filter(|tok| !blank_set.contains(&tok.name_str().as_str()))
    .copied()
    .collect::<Vec<Token>>()
}


pub(crate) fn display_pure_tok(tokens: &[Token]) {
    for token in tokens.iter() {
        println!("{}", token.value_str())
    }
}
