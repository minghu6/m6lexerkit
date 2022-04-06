use m6lexerkit::{
    declare_st, lexdfamap, make_char_matcher_rules, token_recognizer,
    tokenize2, LexDFAMap, SrcFileInfo, TokenizeResult, ENTRY_ST, TokenRecognizer,
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
    antiquote   => "`"               | n,
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
    anybutbslashaq  => r#"[^[`\\]]"# | r,
    anybutstar  => r#"[^[*]]"#       | r,
    newline     => r#"[\n\r]"#       | r,
    anybutnewline => r#"[^[\n\r]]"#  | r,
    x           => "x"               | n,
    alpha       => "[[:alpha:]]"     | r,
    hex         => "[[:xdigit:]]"    | r,
    sharp       => "#"               | n
}

declare_st! {
    BLANK,

    SQUOTE_STR,
    SQUOTE_END,
    SQUOTE_STR_BSLASH,

    DQUOTE_STR,
    DQUOTE_STR_END,
    DQUOTE_STR_BSLASH,

    AQUOTE_STR,
    AQUOTE_STR_END,
    AQUOTE_STR_BSLASH,

    DELIMITER,
    PARENTHESIS,

    IDENT_NAME,
    IDENT_HEAD,

    OP,

    NUM_HEAD,
    NUM_ZERO_HEAD,
    NUM,

    HEX_NUM,

    COMMENT_HEAD,
    BLK_COMMENT,
    BLK_COMMENT_END,
    BLK_COMMENT_END2,
    LN_COMMENT
}


lazy_static! {
    static ref LEX_DFA_MAP: LexDFAMap = lexdfamap! {
        ENTRY_ST => {
            slash       | COMMENT_HEAD_ST,   false
            sp          | BLANK_ST,          false
            ident_head  | IDENT_NAME_ST,     false
            parenthesis | PARENTHESIS_ST,    false
            delimiter   | DELIMITER_ST,      false

            singlequote | SQUOTE_STR_ST,     false
            doublequote | DQUOTE_STR_ST,     false
            antiquote   | AQUOTE_STR_ST,     false

            zero        | NUM_ZERO_HEAD_ST,  false
            num         | NUM_ST,            false
            numsign     | NUM_HEAD_ST,       false
            op          | OP_ST,             false
        },
        BLANK_ST => {
            slash       | COMMENT_HEAD_ST,      true
            sp          | BLANK_ST,             false
            ident_head  | IDENT_NAME_ST,        true
            parenthesis | PARENTHESIS_ST,       true
            delimiter   | DELIMITER_ST,         true
            singlequote | SQUOTE_STR_BSLASH_ST, true
            doublequote | DQUOTE_STR_ST,        true
            antiquote   | AQUOTE_STR_ST,        true
            zero        | NUM_ZERO_HEAD_ST,     true
            num         | NUM_ST,               true
            numsign     | NUM_HEAD_ST,          true
            op          | OP_ST,                true
        },
        IDENT_HEAD_ST => {
            ident_head | IDENT_NAME_ST, false
        },

        SQUOTE_STR_BSLASH_ST => {
            anybutbslashsq | SQUOTE_STR_BSLASH_ST,       false
            bslash         | SQUOTE_STR_BSLASH_ST, false
            singlequote    | SQUOTE_END_ST,       false
        },
        SQUOTE_END_ST => {
            sp          | BLANK_ST,       true
            delimiter   | DELIMITER_ST,   true
            parenthesis | PARENTHESIS_ST, true
        },
        SQUOTE_STR_BSLASH_ST => {
            any | SQUOTE_STR_BSLASH_ST, false
        },

        DQUOTE_STR_ST => {
            anybutbslashdq | DQUOTE_STR_ST,       false
            bslash         | DQUOTE_STR_BSLASH_ST, false
            doublequote    | DQUOTE_STR_END_ST,       false
        },
        DQUOTE_STR_END_ST => {
            sp          | BLANK_ST,       true
            delimiter   | DELIMITER_ST,   true
            parenthesis | PARENTHESIS_ST, true
        },
        DQUOTE_STR_BSLASH_ST => {
            any | DQUOTE_STR_ST, false
        },
        AQUOTE_STR_ST => {
            anybutbslashaq | AQUOTE_STR_ST,       false
            bslash         | AQUOTE_STR_BSLASH_ST, false
            antiquote      | AQUOTE_STR_END_ST,       false
        },
        AQUOTE_STR_END_ST => {
            sp          | BLANK_ST,       true
            delimiter   | DELIMITER_ST,   true
            parenthesis | PARENTHESIS_ST, true
        },
        AQUOTE_STR_BSLASH_ST => {
            any | AQUOTE_STR_ST, false
        },

        DELIMITER_ST => {
            sp         | BLANK_ST, true
            slash      | COMMENT_HEAD_ST, true
            delimiter  | DELIMITER_ST, true
            ident_head | IDENT_NAME_ST, true
        },
        PARENTHESIS_ST => {
            parenthesis | PARENTHESIS_ST,    true
            sp          | BLANK_ST,          true
            delimiter   | DELIMITER_ST,      true
            ident_head  | IDENT_NAME_ST,      true

            singlequote | SQUOTE_STR_BSLASH_ST, true
            doublequote | DQUOTE_STR_ST, true
            antiquote   | AQUOTE_STR_ST, true

            zero        | NUM_ZERO_HEAD_ST,    true
            num         | NUM_ST,            true
            numsign     | NUM_HEAD_ST,        true

            slash       | COMMENT_HEAD_ST,   true

            op          | OP_ST,             true
        },
        IDENT_NAME_ST => {
            ident       | IDENT_NAME_ST, false
            sp          | BLANK_ST,     true
            parenthesis | PARENTHESIS_ST,     true
            delimiter   | DELIMITER_ST, true

            op          | OP_ST,        true
        },
        OP_ST => {
            op          | OP_ST,          false

            zero        | NUM_ZERO_HEAD_ST, true
            num         | NUM_ST,         true
            numsign     | NUM_HEAD_ST,     true

            sp          | BLANK_ST,       true
            ident_head  | IDENT_NAME_ST,   true
            delimiter   | DELIMITER_ST,   true
            parenthesis | PARENTHESIS_ST, true
        },
        NUM_HEAD_ST => {
            zero        | NUM_ZERO_HEAD_ST, false
            num         | NUM_ST,         false
            parenthesis | BLANK_ST,       true
            delimiter   | DELIMITER_ST,   true
            sp          | BLANK_ST,       true
            op          | OP_ST,          true
        },
        NUM_ZERO_HEAD_ST => {
            x           | HEX_NUM_ST,    false
            num         | NUM_ST,       false
            parenthesis | BLANK_ST,     true
            delimiter   | DELIMITER_ST, true
            sp          | BLANK_ST,     true
            op          | OP_ST,        true
        },
        HEX_NUM_ST => {
            hex         | HEX_NUM_ST,    false
            parenthesis | BLANK_ST,     true
            delimiter   | DELIMITER_ST, true
            sp          | BLANK_ST,     true
            op          | OP_ST,        true
        },
        NUM_ST => {
            num         | NUM_ST,       false
            parenthesis | BLANK_ST,     true
            delimiter   | DELIMITER_ST, true
            sp          | BLANK_ST,     true
            op          | OP_ST,        true
        },
        COMMENT_HEAD_ST => {
            slash    | LN_COMMENT_ST,  false
            asterisk | BLK_COMMENT_ST, false
            sp       | BLANK_ST,       true
            eq       | BLANK_ST,       true
        },
        BLK_COMMENT_ST => {
            asterisk   | BLK_COMMENT_END_ST, false
            anybutstar | BLK_COMMENT_ST,    false
        },
        BLK_COMMENT_END_ST => {
            anybutstarslash | BLK_COMMENT_ST,     false
            asterisk        | BLK_COMMENT_END_ST,  false
            slash           | BLK_COMMENT_END2_ST, false
        },
        BLK_COMMENT_END2_ST => {
            sp          | BLANK_ST,          true
            slash       | COMMENT_HEAD_ST,   true
            ident_head  | IDENT_NAME_ST,      true
            delimiter   | DELIMITER_ST,      true
            parenthesis | PARENTHESIS_ST,    true

            singlequote | SQUOTE_STR_BSLASH_ST, true
            doublequote | DQUOTE_STR_ST, true
            antiquote   | AQUOTE_STR_ST, true


            zero        | NUM_ZERO_HEAD_ST,    true
            num         | NUM_ST,            true
            numsign     | NUM_HEAD_ST,        true

            op          | OP_ST,             true
        },
        LN_COMMENT_ST => {
            anybutnewline | LN_COMMENT_ST, false
            newline       | BLANK_ST,     true
        }
    };

    static ref RECOGNIZER: TokenRecognizer = token_recognizer![ 2 |
        id        => "[[:alpha:]_][[:alnum:]_]",

        // Lit
        lit_int   => r"[+|-]?(([0-9]+)|(0x[0-9a-f]+))",
        lit_float => r"[+|-]?([0-9]+\.[0-9])",
        dqstr     => r#"""#,
        aqstr     => r#"`"#,
        lit_regex => r#"/.*/"#,

        // Comment
        slash_line_comment  => r"//",

        // space
        sp      => "[[:BLANK_ST:]]+",
        newline => r#"\n\r?"#,

        // Bracket
        lparen => r"\(",
        rparen => r"\)",
        lbracket => r"\[",
        rbracket => r"\]",
        lbrace => r"\{",
        rbrace => r"\}",

        // DELIMITER
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
    |];
}


/// NEED PreProcess for HereDoc and Regex Literal

#[inline]
pub fn tokenize(source: &SrcFileInfo) -> TokenizeResult {
    tokenize2(&source, &LEX_DFA_MAP, &RECOGNIZER)
}


#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use m6lexerkit::SrcFileInfo;

    use super::tokenize;
    use crate::{display_pure_tok, trim_tokens};

    #[test]
    fn test_tokenize2() {
        let path = PathBuf::from("./examples/exp0.js");
        let srcfile = SrcFileInfo::new(path).unwrap();

        match tokenize(&srcfile) {
            Ok(tokens) => {
                let tokens = trim_tokens(&tokens[..]);
                display_pure_tok(&tokens[..]);
            },
            Err(err) => println!("{}", err),
        }
    }
}
