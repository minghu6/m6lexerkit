use m6tokenizer::{
    declare_st, make_char_matcher_rules,
    tokenize2, SrcFileInfo, TokenizeResult, ENTRY_ST, LexDFAMap, lexdfamap,
};

make_char_matcher_rules! {
    ident       => "[[:alnum:]_]"    | r,
    ident_head  => "[[:alpha:]_]"    | r,
    delimiter   => "[,|;|:]"         | r,
    num         => "[[:digit:]]"     | r,
    numsign     => "[+|-]"           | r,
    op          => r#"[\+|\-|\*|/|%|\^|\||&|~|!|?|@|>|=|<|\.]"# | r,
    any         => r#"[\d\D]"#       | r,
    ng          => r#"[^[:graph:]]"# | r,
    sp          => r#"[[:space:]]"#  | r,
    singlequote => "'"               | n,
    doublequote => "\""              | n,
    slash       => "/"               | n,
    parenthesis => r#"[\[\]{}\(\)]"# | r,
    zero        => "0"               | n,
    bslash      => "\\"              | n,
    question    => "?"               | n,
    eq          => "="               | n,
    lt          => "<"               | n,
    asterisk    => "*"               | n,
    anybutstarslash => r#"[^[*/]]"#  | r,
    anybutbslashsq  => r#"[^['\\]]"# | r,
    anybutbslashdq  => r#"[^["\\]]"# | r,
    anybutstar  => r#"[^[*]]"#       | r,
    newline     => r#"[\n\r]"#       | r,
    anybutnewline => r#"[^[\n\r]]"#  | r,
    x           => "x"               | n,
    hex         => "[[:xdigit:]]"    | r,
    sharp       => "#"               | n
}

declare_st! {
    COMMENT_HEAD
}


lazy_static! {
    static ref LEX_DFA_MAP: LexDFAMap = lexdfamap! {
        ENTRY_ST => {
            slash       | COMMENT_HEAD_ST,   false
            sp          | "Blank",          false
            ident_head  | "IdentName",      false
            parenthesis | "Parenthesis",    false
            delimiter   | "Delimiter",      false
            singlequote | "SingleQuoteStr", false
            doublequote | "DoubleQuoteStr", false
            zero        | "NumZeroHead",    false
            num         | "Num",            false
            numsign     | "NumHead",        false
            op          | "Op",             false
        }
    };
}



#[inline]
pub fn tokenize(source: &SrcFileInfo) -> TokenizeResult {
    tokenize2(&source, &LEX_DFA_MAP)
}


#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use m6tokenizer::SrcFileInfo;

    use super::COMMENT_HEAD_ST;
    use crate::{display_pure_tok, tokenize1, trim_tokens};

    #[test]
    fn test_tokenize2() {
        let path = PathBuf::from("./examples/exp0.js");
        let srcfile = SrcFileInfo::new(path).unwrap();

        println!("{}", COMMENT_HEAD_ST);
        // println!("{:#?}", sp_m(srcfile.get_srcstr(), SrcLoc { ln: 0, col: 0 }));

        // match tokenize1(&srcfile) {
        //     Ok(tokens) => {
        //         let tokens = trim_tokens(&tokens[..]);
        //         display_pure_tok(&tokens[..]);
        //     },
        //     Err(err) => println!("{}", err),
        // }
    }
}
