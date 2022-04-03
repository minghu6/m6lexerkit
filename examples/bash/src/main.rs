use std::path::PathBuf;

use maplit::hashset;

use m6tokenizer::{
    make_token_matcher_rules,
    SrcFileInfo,
    dqstr_m,
    heredoc_m,
    tokenize, TokenMatchResult,
};


make_token_matcher_rules! {
    id        => "[[:alpha:]_][[:alnum:]_]*",
    exec_id   => r"\$[[:alpha:]_][[:alnum:]_]*",
    sharp_line_comment  => r"#.*",

    sp      => "[[:blank:]]+",
    newline => r#"[\n\r]"#,

    heredoc,
    dqstr,
    sqstr => "'[.\n\r]*?'",


    // Operation
    colon => ":",
    lparen => r"\(",
    rparen => r"\)",
    lbracket => r"\[",
    rbracket => r"\]",
    lbrace => r"\{",
    rbrace => r"\}",
    single_arrow  => "->",
    sub    => "-",
    add2   => r"\+\+",
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


fn trim_tokens(tokens: &[Token]) -> Vec<Token> {
    let blank_set = hashset! { "newline", "sp", "sharp_line_comment" };

    tokens
    .iter()
    .filter(|tok| !blank_set.contains(&tok.name_str().as_str()))
    .copied()
    .collect::<Vec<Token>>()
}


fn display_pure_tok(tokens: &[Token]) {
    for token in tokens.iter() {
        println!("{}", token.value_str())
    }
}


fn main() {

    for i in 0..1 {
        let path = PathBuf::from(format!("./examples/exp{}.sh", i));
        let srcfile = SrcFileInfo::new(path).unwrap();

        // println!("{:#?}", sp_m(srcfile.get_srcstr(), SrcLoc { ln: 0, col: 0 }));

        match tokenize(&srcfile, &MATCHERS[..]) {
            Ok(tokens) => {
                let trimed_tokens = trim_tokens(&tokens[..]);
                display_pure_tok(&trimed_tokens[..]);
            },
            Err(err) => println!("{}", err),
        }
    }

}
