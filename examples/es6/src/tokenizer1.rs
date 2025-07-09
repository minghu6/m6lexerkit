use m6lexerkit::{
    SrcFileInfo, TokenMatchResult, TokenizeResult, make_token_matcher_rules,
    prelude::{aqstr_m, dqstr_m, lit_regex_m, sqstr_m},
    tokenize as tokenize_,
};


make_token_matcher_rules! {
    id        => "[[:alpha:]_][[:alnum:]_]*",

    // Lit
    lit_int => r"[+|-]?(([0-9]+)|(0x[0-9a-f]+))",
    lit_float => r"[+|-]?([0-9]+\.[0-9])",
    sqstr,
    dqstr,
    aqstr,
    lit_regex,

    // Comment
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


#[inline]
pub fn tokenize(source: &SrcFileInfo) -> TokenizeResult {
    tokenize_(source, &MATCHERS[..])
}



#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use m6lexerkit::SrcFileInfo;

    use crate::{display_pure_tok, tokenize1, trim_tokens};


    #[test]
    fn test_tokenize1() {
        use std::thread::spawn;

        let t1 = spawn(|| {
            let path = PathBuf::from("./resources/exp0.js");
            let srcfile = SrcFileInfo::new(&path).unwrap();

            match tokenize1(&srcfile) {
                Ok(tokens) => {
                    let tokens = trim_tokens(&tokens[..]);
                    display_pure_tok(&tokens[..]);
                }
                Err(err) => println!("{}", err),
            }
        });

        let path = PathBuf::from("./resources/exp0.js");
        let srcfile = SrcFileInfo::new(&path).unwrap();

        match tokenize1(&srcfile) {
            Ok(tokens) => {
                let tokens = trim_tokens(&tokens[..]);
                display_pure_tok(&tokens[..]);
            }
            Err(err) => println!("{}", err),
        }

        t1.join().unwrap();

    }
}
