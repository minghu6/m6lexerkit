use std::path::PathBuf;

use maplit::hashset;

use m6tokenizer::{
    make_token_matcher_rules,
    SrcFileInfo,
    dqstr_m,
    aqstr_m,
    lit_regex_m,
    tokenize, TokenMatchResult,
};


make_token_matcher_rules! {
    id        => "[[:alpha:]_][[:alnum:]_]*",

    // Lit
    lit_int => r"[+|-]?(([0-9]+)|(0x[0-9a-f]+))",
    lit_float => r"[+|-]?([0-9]+\.[0-9])",
    dqstr,
    aqstr,
    lit_regex,

    // Comment
    exec_id   => r"\$[[:alpha:]_][[:alnum:]_]*",
    slash_line_comment  => r"//.*",

    // space
    sp      => "[[:blank:]]+",
    newline => r#"\n\r?"#,

    // Bracket
    lparen => r"\(",
    rparen => r"\)",
    lbracket => r"\[",
    rbracket => r"\]",
    lbrace => r"\{",
    rbrace => r"\}",

    // Delimiter
    colon => ":",
    question => r"\?",
    double_arrow  => "=>",
    semi   => ";",
    comma  => ",",

    // Assign
    assign => "=",

    // Unary Operation
    inc => r"\+\+",
    dec => r"--",
    not => "!",

    // Binary Operation
    sub    => "-",
    add    => r"\+[^\+]",
    mul    => r"\*",
    div    => "/",
    dot    => r"\.",
    ge     => ">=",
    le     => "<=",
    lt     => "<",
    gt     => ">",
    realeq => "===",
    nrealeq => "!==",
    neq    => "!=",
    eq     => "==",
    percent=> "%",
    and    => "&&",
    or     => r"\|\|"

}


fn trim_tokens(tokens: &[Token]) -> Vec<Token> {
    let blank_set = hashset! { "newline", "sp", "slash_line_comment" };

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
    // let res = lit_regex_m(r#"/[|\\{}()[\]^$+*?.]/g"#, 0);
    // println!("res: {:?}", res);

    for i in 0..1 {
        let path = PathBuf::from(format!("./examples/exp{}.js", i));
        let srcfile = SrcFileInfo::new(path).unwrap();

        // println!("{:#?}", sp_m(srcfile.get_srcstr(), SrcLoc { ln: 0, col: 0 }));

        match tokenize(&srcfile, &MATCHERS[..]) {
            Ok(tokens) => {
                let tokens = trim_tokens(&tokens[..]);
                display_pure_tok(&tokens[..]);
            },
            Err(err) => println!("{}", err),
        }
    }

}
