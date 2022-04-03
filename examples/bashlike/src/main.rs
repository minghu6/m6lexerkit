
use std::path::PathBuf;

use m6tokenizer::{
    make_token_matcher_rules,
    SrcFileInfo,
    aux_strlike_m,
    tokenize, TokenMatchResult,
};


make_token_matcher_rules! {
    ellipsis2 => "\\.\\.",

    id     => "[[:alpha:]_][[:alnum:]_]*",
    lit_int => r"[+|-]?(([0-9]+)|(0x[0-9a-f]+))",
    lit_float => r"[+|-]?([0-9]+\.[0-9])",

    cmd,
    slash_block_comment => r"/\*.*",
    slash_line_comment  => r"//.*",

    sp      => "[[:space:]--[\n\r]]+",
    newline => r#"[\n\r]"#,

    dqstr => r#"^"[\s\S]*""#,

    colon => ":",
    lparen => r"\(",
    rparen => r"\)",
    lbracket => r"\[",
    rbracket => r"\]",
    lbrace => r"\{",
    rbrace => r"\}",
    single_arrow  => "->",
    sub    => "-",
    add2    => r"\+\+",
    add    => r"\+",
    mul    => r"\*",
    div    => "/",
    semi   => ";",
    dot    => r"\.",
    comma  => ",",
    lt     => "<",
    eq     => "=",
    percent=> "%"
}


fn cmd_m(source: &str, from: usize) -> Option<TokenMatchResult> {
    aux_strlike_m(source, from, "!(", ")", '\\')
    .and_then(|res|
        Some(res.and_then(|tok| Ok(tok.rename("cmd"))))
    )
}



fn main() {

    for i in 0..1 {
        let path = PathBuf::from(format!("./examples/exp{}", i));
        let srcfile = SrcFileInfo::new(path).unwrap();

        // println!("{:#?}", sp_m(srcfile.get_srcstr(), SrcLoc { ln: 0, col: 0 }));

        match tokenize(&srcfile, &MATCHERS[..]) {
            Ok(tokens) => println!("{:#?}", tokens),
            Err(err) => println!("{}", err),
        }
    }

}
