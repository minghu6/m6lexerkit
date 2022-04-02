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



////////////////////////////////////////////////////////////////////////////////
//// Token

#[derive(Clone)]
pub struct Token {
    pub name: Symbol,
    pub value: Symbol,
    pub loc: SrcLoc,
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
    pub fn bytes_len(&self) -> usize {
        INTERNER.with(|interner| {
            interner.borrow().resolve(self.value).unwrap().len()
        })
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "name: <{}>", self.name_str(),)?;
        writeln!(f, "value: {}", self.value_str(),)?;

        writeln!(f, "loc: {}", self.loc)?;
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

    pub fn fetch_tok(&self, text: &str, loc: SrcLoc) -> Option<TokenMatchResult> {
        self.pat
            .captures_read(&mut self.pat.capture_locations(), text)
            .and_then(|mat| {
                Some(Ok(Token {
                    name: self.tok_name,
                    value: str2sym(mat.as_str()),
                    loc,
                }))
            })
    }
}

pub type FnMatcher = fn(&str, SrcLoc) -> Option<TokenMatchResult>;



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
        let loc = srcfile.offset2srcloc(chars_pos);

        for fn_matcher in fn_matchers.iter() {
            if let Some(tokres) = fn_matcher(&source[bytes_pos..], loc) {
                match tokres {
                    Ok(tok) => {
                        chars_pos += tok.chars_len();
                        bytes_pos += tok.bytes_len();

                        tokens.push(tok);
                        tok_matched = true;
                        break;
                    },
                    Err(reason) => {
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
    prefix: &str,
    postfix: &str,
    escape_char: char,
) -> Option<Result<Symbol, TokenizeErrorReason>> {
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
                    val.push(if c == mat {
                        mat
                    } else {
                        return Some(Err(
                            TokenizeErrorReason::UnexpectedPostfix,
                        ));
                    });
                }
                else {
                    break;
                }
            }
            _ => unreachable!(),
        }
    }

    Some(Ok(
        str2sym(&(prefix.to_owned() + &val))
    ))
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
