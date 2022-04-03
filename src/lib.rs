use std::{cell::RefCell, error::Error, fmt, fs, path::PathBuf};

pub use lazy_static;
// pub use crate::lexer::{
//     Tokenizer,
//     RegexCharMatcher,
//     LexDFAMapType,
//     ST_ENTRY
// };
pub use proc_macros::make_token_matcher_rules;
use regex::Regex;
use fancy_regex::Regex as RegexEh;
use string_interner::{symbol::DefaultSymbol, StringInterner};

thread_local! {
    pub static INTERNER: RefCell<StringInterner> = RefCell::new(StringInterner::default());
}

pub type Symbol = DefaultSymbol;


////////////////////////////////////////////////////////////////////////////////
//// Source File Structure

/// SrcFileInfo
#[allow(dead_code)]
#[derive(PartialEq, Eq)]
pub struct SrcFileInfo {
    /// Source file path
    path: PathBuf,

    /// lines[x]: number of total chars until lines x [x]
    /// inspired by `proc_macro2`: `FileInfo`
    lines: Vec<usize>,

    srcstr: String,
}

impl SrcFileInfo {
    pub fn new(path: PathBuf) -> Result<Self, Box<dyn Error>> {
        let srcstr = fs::read_to_string(&path)?;

        let lines = Self::build_lines(&srcstr);

        Ok(Self {
            path,
            lines,
            srcstr,
        })
    }

    fn build_lines(srcstr: &str) -> Vec<usize> {
        let mut lines = vec![0];
        let mut total = 0usize;

        for c in srcstr.chars() {
            total += 1;

            if c == '\n' {
                lines.push(total);
            }
        }

        lines
    }

    pub fn get_srcstr(&self) -> &str {
        &self.srcstr
    }

    pub fn get_path(&self) -> &PathBuf {
        &self.path
    }

    pub fn offset2srcloc(&self, offset: usize) -> SrcLoc {
        match self.lines.binary_search(&offset) {
            Ok(found) => {
                SrcLoc {
                    ln: found + 1,
                    col: 1, // 换行处
                }
            }
            Err(idx) => {
                SrcLoc {
                    ln: idx,
                    col: offset - self.lines[idx - 1] + 1, // 显然idx >= 0
                }
            }
        }
    }

    pub fn filename(&self) -> String {
        self.path.file_name().unwrap().to_string_lossy().to_string()
    }

    pub fn dirname(&self) -> String {
        self.path.parent().unwrap().to_string_lossy().to_string()
    }
}

impl fmt::Debug for SrcFileInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SrcFileInfo")
            .field("path", &self.path)
            .finish()
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd)]
pub struct SrcLoc {
    pub ln: usize,
    pub col: usize,
}

impl SrcLoc {
    pub fn new(loc_tuple: (usize, usize)) -> Self {
        Self {
            ln: loc_tuple.0,
            col: loc_tuple.1,
        }
    }
}

impl fmt::Debug for SrcLoc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl fmt::Display for SrcLoc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.ln, self.col)
    }
}

impl Ord for SrcLoc {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.ln == other.ln {
            self.col.cmp(&other.col)
        } else {
            self.ln.cmp(&other.ln)
        }
    }
}


#[derive(Clone, Copy)]
pub struct Span {
    pub from: usize,  // bytes offset used for index from origin file
    pub end: usize
}

impl Span {
    #[inline]
    pub fn len(&self) -> usize {
        self.end - self.from
    }
}



////////////////////////////////////////////////////////////////////////////////
//// Token

#[derive(Clone, Copy)]
pub struct Token {
    pub name: Symbol,
    pub value: Symbol,
    pub span: Span,
}

impl Token {
    pub fn name_str(&self) -> String {
        sym2str(self.name)
    }

    pub fn value_str(&self) -> String {
        sym2str(self.value)
    }

    /// value's chars len
    #[inline]
    pub fn chars_len(&self) -> usize {
        INTERNER.with(|interner| {
            interner
                .borrow()
                .resolve(self.value)
                .unwrap()
                .chars()
                .count()
        })
    }

    /// value's bytes len
    #[inline]
    pub fn span_len(&self) -> usize {
        self.span.len()
    }

    pub fn rename(self, name: &str) -> Self {
        Self {
            name: str2sym(name),
            value: self.value,
            span: self.span,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "name: <{}>", self.name_str(),)?;
        writeln!(f, "value: {}", self.value_str(),)?;
        writeln!(f, "len: {}", self.chars_len())
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}


////////////////////////////////////////////////////////////////////////////////
//// Tokenize

pub struct TokenMatcher {
    pat: Regex,
    tok_name: Symbol,
}

impl TokenMatcher {
    pub fn new(patstr: &str, tok_name: &str) -> Self {
        Self {
            pat: Regex::new(patstr).unwrap(),
            tok_name: str2sym(tok_name),
        }
    }

    pub fn fetch_tok(&self, text: &str, start: usize) -> Option<TokenMatchResult> {
        self.pat
            .captures(text)
            .and_then(|cap| {
                let bytes_len = cap.get(0).unwrap().as_str().len();
                let mat = cap.get(1).unwrap().as_str();

                Some(Ok(Token {
                    name: self.tok_name,
                    value: str2sym(mat),
                    span: Span { from: start, end: start + bytes_len },
                }))
            })
    }
}

pub type FnMatcher = fn(&str, usize) -> Option<TokenMatchResult>;



#[derive(Debug)]
pub enum TokenizeErrorReason {
    UnrecognizedToken,
    UnrecognizedEscaped(char),
    UnexpectedPostfix
}


#[allow(unused)]
#[derive(Debug)]
pub struct TokenizeError {
    reason: TokenizeErrorReason,
    loc: SrcLoc,
    path: PathBuf,
}
impl std::error::Error for TokenizeError {}
impl std::fmt::Display for TokenizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#?}", self)
    }
}


pub type TokenizeResult = Result<Vec<Token>, TokenizeError>;
pub type TokenMatchResult = Result<Token, TokenizeErrorReason>;

pub fn tokenize(
    srcfile: &SrcFileInfo,
    fn_matchers: &[FnMatcher],
) -> TokenizeResult {
    let source = srcfile.get_srcstr();
    let mut tokens = vec![];

    if source.len() == 0 {
        return Ok(tokens);
    }

    let mut bytes_pos = 0;
    let mut chars_pos = 0;

    while bytes_pos < source.len() {
        let mut tok_matched = false;

        for fn_matcher in fn_matchers.iter() {
            if let Some(tokres) = fn_matcher(&source[bytes_pos..], bytes_pos) {
                match tokres {
                    Ok(tok) => {
                        chars_pos += tok.chars_len();
                        bytes_pos += tok.span_len();

                        tokens.push(tok);
                        tok_matched = true;
                        break;
                    },
                    Err(reason) => {
                        let loc = srcfile.offset2srcloc(chars_pos);

                        return Err(TokenizeError {
                            reason,
                            loc,
                            path: srcfile.path.clone(),
                        });
                    },
                }
            }
        }

        if !tok_matched {
            let loc = srcfile.offset2srcloc(chars_pos);

            return Err(TokenizeError {
                reason: TokenizeErrorReason::UnrecognizedToken,
                loc,
                path: srcfile.path.clone(),
            });
        }
    }

    Ok(tokens)
}



////////////////////////////////////////////////////////////////////////////////
//// Auxiliary

pub fn sym2str(sym: Symbol) -> String {
    INTERNER.with(|interner| {
        interner.borrow().resolve(sym).unwrap().to_owned()
    })
}

pub fn str2sym(s: &str) -> Symbol {
    INTERNER.with(|interner| {
        interner.borrow_mut().get_or_intern(s)
    })
}


///
/// handle this token type:
///
/// 1. contains anything but delimiter
/// 1. delimiter can be escaped char
///
pub fn aux_strlike_m(
    source: &str,
    from: usize,
    prefix: &str,
    postfix: &str,
    escape_char: char,
) -> Option<Result<Token, TokenizeErrorReason>> {
    debug_assert!(!prefix.is_empty());
    debug_assert!(!postfix.is_empty());

    if !source.starts_with(prefix) {
        return None;
    }

    let mut postfix_iter = postfix.chars().into_iter();
    let delimiter = postfix_iter.next().unwrap();
    let mut val = String::new();

    let mut st = 0;
    // st:
    //    0 normal mode
    //    1 escape mode
    //    2 tail mode  // collect tail

    for c in source[prefix.len()..].chars() {
        match st {
            0 => {
                if c == escape_char {
                    st = 1;
                } else if c == delimiter {
                    st = 2;
                    val.push(c);
                } else {
                    val.push(c);
                }
            }
            1 => {
                st = 0;
                val.push(if c == delimiter {
                    delimiter
                } else if c == escape_char {
                    escape_char
                } else {
                    return Some(Err(
                        TokenizeErrorReason::UnrecognizedEscaped(c),
                    ));
                });
            }
            2 => {
                if let Some(mat) = postfix_iter.next() {
                    if c != mat {
                        return Some(Err(
                            TokenizeErrorReason::UnexpectedPostfix,
                        ))
                    }
                }
                else {
                    break;
                }
            }
            _ => unreachable!(),
        }
    }

    let span_len = prefix.len() + val.len() + postfix.len();
    let span = Span { from, end: from + span_len };
    let value = str2sym(&val);
    let name = str2sym("__aux_tmp");

    Some(Ok(Token {
        name,
        value,
        span,
    }))
}

/// Double quote string
#[inline]
pub fn dqstr_m(
    source: &str,
    from: usize
) -> Option<TokenMatchResult> {
    aux_strlike_m(source, from, "\"", "\"", '\\')
    .and_then(|res|
        Some(res.and_then(|tok| Ok(tok.rename("dqstr"))))
    )
}

/// Double quote string
#[inline]
pub fn aqstr_m(
    source: &str,
    from: usize
) -> Option<TokenMatchResult> {
    aux_strlike_m(source, from, "`", "`", '\\')
    .and_then(|res|
        Some(res.and_then(|tok| Ok(tok.rename("aqstr"))))
    )
}

/// Single quote string
#[inline]
pub fn sqstr_m(
    source: &str,
    from: usize
) -> Option<TokenMatchResult> {
    aux_strlike_m(source, from, "'", "'", '\\')
    .and_then(|res|
        Some(res.and_then(|tok| Ok(tok.rename("sqstr"))))
    )
}


///
/// handle this heredoc:
pub fn heredoc_m(
    source: &str,
    from: usize,
) -> Option<Result<Token, TokenizeErrorReason>> {

    lazy_static::lazy_static! {
        pub static ref HEREDOC_2_REG_EH: RegexEh = RegexEh::new(
            r#"^(<<<|<<-|<<|<-)[[:blank:]]*(.+)([[:blank:]]+.*\n|\n)([\s|\S]*?)\n\2"#
        ).unwrap();
    }

    let cap_opt = HEREDOC_2_REG_EH.captures(source).unwrap();

    if let Some(cap) = cap_opt {
        let bytes_len = cap.get(0).unwrap().as_str().len();
        let value = str2sym(cap.get(4).unwrap().as_str());
        let span = Span { from, end: from + bytes_len };
        let name = str2sym("__aux_tmp");

        Some(Ok(Token {
            name,
            value,
            span,
        }))
    }
    else {
        None
    }
}



#[cfg(test)]
mod tests {

    #[test]
    fn test_strmatch() {
        let s = "语言特定的函数(以下也称为routine, 名字用`__personality_routine`指代), 用于和**unwinding library** 配合做语言特定的异常处理";

        let mut p = 0;
        let end = s.len();

        while p < end {
            let new_s = &s[p..end];
            println!("{}", new_s);
            p += 3;
        }
    }
}
