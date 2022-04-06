#![feature(test)]

mod tokenizer1;
mod tokenizer2;

use m6lexerkit::Token;
use maplit::hashset;

pub use tokenizer1::tokenize as tokenize1;
pub use tokenizer2::tokenize as tokenize2;

#[allow(unused)]
pub(crate) fn trim_tokens(tokens: &[Token]) -> Vec<Token> {
    let blank_set = hashset! { "newline", "sp", "slash_line_comment" };

    tokens
    .iter()
    .filter(|tok| !blank_set.contains(&tok.name_str().as_str()))
    .copied()
    .collect::<Vec<Token>>()
}

#[allow(unused)]
pub(crate) fn display_pure_tok(tokens: &[Token]) {
    for token in tokens.iter() {
        println!("{}", token.value_str())
    }
}


#[cfg(test)]
mod tests {

    extern crate test;
    use std::path::PathBuf;

    use m6lexerkit::SrcFileInfo;
    use test::Bencher;

    use crate::tokenize1;
    use crate::tokenize2;

    #[bench]
    fn bench_tokenizer1(b: &mut Bencher) {
        let srcfile
        = SrcFileInfo::new(PathBuf::from("./examples/app.js")).unwrap();

        b.iter(|| {
            let _tokens = tokenize1(&srcfile).unwrap();
        });
    }

    #[bench]
    fn bench_tokenizer2(b: &mut Bencher) {
        let srcfile
        = SrcFileInfo::new(PathBuf::from("./examples/app.js")).unwrap();

        b.iter(|| {
            let _tokens = tokenize2(&srcfile).unwrap();

            // Mock PreProcess
            for c in srcfile.get_srcstr().chars() {
                let _ = c.to_owned();
            }

            for c in srcfile.get_srcstr().chars() {
                let _ = c.to_owned();
            }
        });
    }

}