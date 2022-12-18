use std::{
    cell::RefCell,
    cmp::min,
    collections::HashMap,
    error::Error,
    fmt, fs,
    hash::Hash,
    path::{Path, PathBuf},
};

pub use concat_idents::concat_idents as concat_idents2;
pub use lazy_static;
pub use proc_macros::{make_char_matcher_rules, make_token_matcher_rules};
pub use regex::Regex;
use string_interner::{symbol::DefaultSymbol, StringInterner};

thread_local! {
    pub static INTERNER: RefCell<StringInterner> = RefCell::new(StringInterner::default());
}

// pub type Symbol = DefaultSymbol;

#[derive(Clone, Copy)]
pub struct Symbol(pub DefaultSymbol);

impl std::fmt::Debug for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", sym2str(*self))
    }
}

impl Hash for Symbol {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl PartialEq for Symbol {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for Symbol {}

impl PartialOrd for Symbol {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Ord for Symbol {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}



////////////////////////////////////////////////////////////////////////////////
//// Source File Structure

/// SrcFileInfo
#[allow(dead_code)]
#[derive(PartialEq, Eq, Clone)]
pub struct SrcFileInfo {
    /// Source file path
    path: PathBuf,

    /// lines[x]: number of total chars until lines x [x]
    /// inspired by `proc_macro2`: `FileInfo`
    lines: Vec<usize>,
    blines: Vec<usize>, // bytes offset

    srcstr: String,
}

impl SrcFileInfo {
    pub fn new<P: AsRef<Path>>(path: &P) -> Result<Self, Box<dyn Error>> {
        let srcstr = fs::read_to_string(&path)?;
        let path = path.as_ref().to_owned();

        let lines = Self::build_lines(&srcstr);
        let blines = Self::build_blines(&srcstr);

        Ok(Self {
            path,
            lines,
            blines,
            srcstr,
        })
    }

    pub fn from_str(srcstr: String) -> Self {
        let lines = Self::build_lines(&srcstr);
        let blines = Self::build_blines(&srcstr);

        Self {
            path: PathBuf::new(),
            lines,
            blines,
            srcstr,
        }
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

    fn build_blines(srcstr: &str) -> Vec<usize> {
        let mut lines = vec![0];
        let mut total = 0usize;

        for c in srcstr.bytes() {
            total += 1;

            if c == b'\n' {
                lines.push(total);
            }
        }

        lines
    }

    pub fn get_srcstr(&self) -> &str {
        &self.srcstr
    }

    pub fn get_path(&self) -> &Path {
        &self.path.as_path()
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

    /// bytes offset
    pub fn boffset2srcloc(&self, offset: usize) -> SrcLoc {
        match self.blines.binary_search(&offset) {
            Ok(found) => {
                SrcLoc {
                    ln: found + 1,
                    col: 1, // 换行处
                }
            }
            Err(idx) => {
                SrcLoc {
                    ln: idx,
                    col: self.srcstr[self.blines[idx - 1]..offset]
                        .chars()
                        .count()
                        + 1, // 显然idx >= 0
                }
            }
        }
    }

    pub fn linestr(&self, cur: usize) -> Option<&str> {
        let SrcLoc { ln, col: _ } = self.boffset2srcloc(cur);

        if ln - 1 >= self.blines.len() {
            None
        } else {
            let start = self.blines[ln - 1];
            let end;
            if ln == self.blines.len() {
                self.srcstr.get(start..)
            } else {
                end = self.blines[ln];
                self.srcstr.get(start..end)
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


#[derive(
    Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Default, Debug,
)]
pub struct Span {
    pub from: usize, // bytes offset used for index from origin file
    pub end: usize,
}

impl Span {
    #[inline]
    pub fn len(&self) -> usize {
        self.end - self.from
    }

    pub fn chars_count(&self, source: &str) -> usize {
        source[self.from..self.end].chars().count()
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
    pub fn eof() -> Self {
        Self {
            name: str2sym("eof"),
            value: str2sym(""),
            span: Span::default(),
        }
    }

    pub fn name_string(&self) -> String {
        sym2str(self.name)
    }

    pub fn value_string(&self) -> String {
        sym2str(self.value)
    }

    /// value's chars len
    #[inline]
    pub fn chars_len(&self) -> usize {
        INTERNER.with(|interner| {
            interner
                .borrow()
                .resolve(self.value.0)
                .unwrap()
                .chars()
                .count()
        })
    }

    pub fn span(&self) -> Span {
        self.span
    }

    /// value's bytes len
    #[inline]
    pub fn span_len(&self) -> usize {
        self.span().len()
    }

    #[inline]
    pub fn span_chars_count(&self, source: &str) -> usize {
        self.span().chars_count(source)
    }

    pub fn rename(self, name: &str) -> Self {
        Self {
            name: str2sym(name),
            value: self.value,
            span: self.span,
        }
    }

    pub fn mapval(self, val: &str) -> Self {
        Self {
            name: self.name,
            value: str2sym(val),
            span: self.span,
        }
    }

    pub fn rename_by_value(self, values: &[&str]) -> Self {
        for value in values.into_iter() {
            if self.check_value(*value) {
                return self.rename(*value);
            }
        }
        self
    }

    pub fn check_value(&self, value: &str) -> bool {
        INTERNER.with(|internner| {
            internner.borrow().resolve(self.value.0).unwrap() == value
        })
    }

    pub fn check_name(&self, name: &str) -> bool {
        INTERNER.with(|internner| {
            internner.borrow().resolve(self.name.0).unwrap() == name
        })
    }

    pub fn check_names_in(&self, targets: &[&str]) -> bool {
        INTERNER.with(|internner| {
            let internref = internner.borrow();
            let name = internref.resolve(self.name.0).unwrap();

            targets
                .into_iter()
                .find(|&&target| target == name)
                .is_some()
        })
    }

    pub fn check_values_in(&self, targets: &[&str]) -> bool {
        INTERNER.with(|internner| {
            let internref = internner.borrow();
            let value = internref.resolve(self.value.0).unwrap();

            targets
                .into_iter()
                .find(|&&target| target == value)
                .is_some()
        })
    }
}


impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "name: <{}>", self.name_string(),)?;
        writeln!(f, "value: {}", self.value_string(),)?;
        writeln!(f, "len: {}", self.chars_len())
    }
}

impl fmt::Debug for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Token")
            .field("name", &self.name_string())
            .field("value", &self.value_string())
            .finish()
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

    pub fn fetch_tok(
        &self,
        text: &str,
        start: usize,
    ) -> Option<TokenMatchResult> {
        self.pat.captures(text).and_then(|cap| {
            let bytes_len = cap.get(0).unwrap().as_str().len();
            let mat = cap.get(1).unwrap().as_str();
            let span = Span {
                from: start,
                end: start + bytes_len,
            };

            Some(Ok(Token {
                name: self.tok_name,
                value: str2sym(mat),
                span,
            }))
        })
    }
}

pub type FnMatcher = fn(&str, usize) -> Option<TokenMatchResult>;



#[derive(Debug)]
pub enum TokenizeErrorReason {
    UnrecognizedToken,
    UnrecognizedEscaped(char),
    UnexpectedPostfix,
    ZeroLenToken
}


pub struct TokenizeError {
    reason: TokenizeErrorReason,
    start: usize,
    src: SrcFileInfo,
}
impl std::error::Error for TokenizeError {}
impl std::fmt::Display for TokenizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let linestr = self.src.linestr(self.start).unwrap();
        let loc = self.src.offset2srcloc(self.start);
        let rem_len = linestr.len() - loc.col;

        writeln!(f)?;
        writeln!(f)?;

        writeln!(f, "{:?}:", self.reason)?;

        writeln!(f)?;
        writeln!(f, "{linestr}")?;
        writeln!(
            f,
            "{}{}{}",
            " ".repeat(loc.col - 1),
            '^',
            "-".repeat(rem_len)
        )?;
        writeln!(
            f,
            "--> {}:{}:{}",
            self.src.get_path().to_string_lossy(),
            loc.ln,
            loc.col
        )?;
        writeln!(f)?;


        Ok(())
    }
}
impl std::fmt::Debug for TokenizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
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
    let mut chars_pos = 0usize;

    while bytes_pos < source.len() {
        let mut tok_matched = false;

        for fn_matcher in fn_matchers.iter() {
            if let Some(tokres) = fn_matcher(&source[bytes_pos..], bytes_pos) {
                match tokres {
                    Ok(tok) => {
                        if tok.span_len() == 0 {
                            return Err(TokenizeError {
                                reason: TokenizeErrorReason::ZeroLenToken,
                                start: chars_pos,
                                src: srcfile.clone(),
                            })
                        }

                        chars_pos += tok.span_chars_count(source);
                        bytes_pos += tok.span_len();

                        tokens.push(tok);
                        tok_matched = true;
                        break;
                    }
                    Err(reason) => {
                        return Err(TokenizeError {
                            reason,
                            start: chars_pos,
                            src: srcfile.clone(),
                        });
                    }
                }
            }
        }

        if !tok_matched {
            return Err(TokenizeError {
                reason: TokenizeErrorReason::UnrecognizedToken,
                start: chars_pos,
                src: srcfile.clone(),
            });
        }
    }

    Ok(tokens)
}



////////////////////////////////////////////////////////////////////////////////
//// Auxiliary

pub fn sym2str(sym: Symbol) -> String {
    INTERNER
        .with(|interner| interner.borrow().resolve(sym.0).unwrap().to_owned())
}

pub fn str2sym(s: &str) -> Symbol {
    Symbol(INTERNER.with(|interner| interner.borrow_mut().get_or_intern(s)))
}



pub mod prelude {
    use std::collections::HashSet;

    use fancy_regex::Regex as RegexEh;

    use proc_macros::make_token_matcher_rules;

    use crate::{str2sym, Span, TokenizeErrorReason, TokenMatchResult, TokenizeResult};


    pub fn trim(res: TokenizeResult) -> TokenizeResult {
        res.and_then(|toks| {Ok(
            toks
            .into_iter()
            .filter(|tok| {
                if tok.check_names_in(&[
                    "newline",
                    "sp",
                    "sharp_line_comment",
                    "slash_line_comment"
                ])
                {
                    false
                }
                else {
                    true
                }})
            .collect::<Vec<Token>>()
        )})
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
                    }
                    val.push(c);
                }
                1 => {
                    st = 0;
                    val.push(c);
                }
                2 => {
                    if let Some(mat) = postfix_iter.next() {
                        if c != mat {
                            return Some(Err(
                                TokenizeErrorReason::UnexpectedPostfix,
                            ));
                        }
                    } else {
                        break;
                    }
                }
                _ => unreachable!(),
            }
        }
        val.pop().unwrap(); // pop delimiter

        let span_len = prefix.len() + val.len() + postfix.len();
        let span = Span {
            from,
            end: from + span_len,
        };
        let value = str2sym(&val);
        let name = str2sym("__aux_tmp");

        Some(Ok(Token { name, value, span }))
    }

    /// Double quote string
    #[inline]
    pub fn dqstr_m(source: &str, from: usize) -> Option<TokenMatchResult> {
        aux_strlike_m(source, from, "\"", "\"", '\\')
            .and_then(|res| Some(res.and_then(|tok| Ok(tok.rename("dqstr")))))
    }

    /// Double quote string
    #[inline]
    pub fn aqstr_m(source: &str, from: usize) -> Option<TokenMatchResult> {
        aux_strlike_m(source, from, "`", "`", '\\')
            .and_then(|res| Some(res.and_then(|tok| Ok(tok.rename("aqstr")))))
    }

    /// Single quote string
    #[inline]
    pub fn sqstr_m(source: &str, from: usize) -> Option<TokenMatchResult> {
        aux_strlike_m(source, from, "'", "'", '\\')
            .and_then(|res| Some(res.and_then(|tok| Ok(tok.rename("sqstr")))))
    }

    #[inline]
    pub fn lit_regex_m(source: &str, from: usize) -> Option<TokenMatchResult> {
        aux_strlike_m(source, from, "/", "/", '\\')
    .and_then(|res|
        match res {
            Ok(mut tok) => {
                lazy_static::lazy_static! {
                    pub static ref ALPHABET__: HashSet<char> = ('a'..='z').chain('A'..='Z').collect();
                }

                let mut tokv = tok.value_string();
                if tokv.is_empty() {  // It' maybe slash comment
                    return None;
                }

                tokv.insert(0, '/');
                tokv.push('/');

                if let Some(nxtc) = source[tok.span_len()..].chars().next() {
                    if ALPHABET__.contains(&nxtc) {
                        tokv.push(nxtc);

                        let span = Span {
                            from,
                            end: from + tok.span_len() + nxtc.to_string().len(),
                        };

                        tok.span = span;
                    }
                }

                tok.value = str2sym(&tokv);

                Some(Ok(tok.rename("lit_regex")))
            },
            _ => unreachable!(),
        }
    )
    }


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
            let span = Span {
                from,
                end: from + bytes_len,
            };

            let value = str2sym(cap.get(4).unwrap().as_str());
            let name = str2sym("__aux_tmp");

            Some(Ok(Token { name, value, span }))
        } else {
            None
        }
    }

    use crate as m6lexerkit;

    make_token_matcher_rules! {
        // Comment
        sharp_line_comment  => r"#.*",

        // White characters
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
        rarrow => "->",
        rdarrow  => "=>",
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
        neq    => "!=",
        eq     => "==",
        percent=> "%",
        and    => "&&",
        or     => r"\|\|"
    }
}


////////////////////////////////////////////////////////////////////////////////
//// Tokenizer2

////////////////////////////////////////////////////////////////////////////////
//// Char Matcher (used for string splitter)

pub trait CharMatcher {
    fn is_match(&self, c: char) -> bool;
}

/// Simple Char Matcher
pub struct SimpleCharMatcher {
    target: char,
}

impl SimpleCharMatcher {
    pub fn new(s: &str) -> Self {
        Self {
            target: s.chars().nth(0).unwrap(),
        }
    }
}

impl CharMatcher for SimpleCharMatcher {
    #[inline]
    fn is_match(&self, c: char) -> bool {
        self.target == c
    }
}

pub struct RegexCharMatcher {
    pat: Regex,
}

impl RegexCharMatcher {
    pub fn new(patstr: &str) -> Self {
        Self {
            pat: Regex::new(patstr).unwrap(),
        }
    }
}

impl CharMatcher for RegexCharMatcher {
    #[inline]
    fn is_match(&self, c: char) -> bool {
        self.pat.is_match(&c.to_string())
    }
}

pub type FnCharMatcher = fn(char) -> bool;
pub type LexDFAMap = HashMap<Symbol, Vec<(FnCharMatcher, (Symbol, bool))>>;

#[allow(unused)]
pub const ENTRY_ST: &'static str = "Entry";

pub struct LexDFA<'a> {
    map: &'a LexDFAMap,
    st: Symbol,
}

impl<'a> LexDFA<'a> {
    pub fn new(map: &'a LexDFAMap) -> Self {
        Self {
            map,
            st: str2sym(ENTRY_ST),
        }
    }

    // Token END?
    pub fn forward(&mut self, ch: char) -> bool {
        let items = self.map.get(&self.st).unwrap();

        for (matcher, (sym, res)) in items.into_iter() {
            if matcher(ch) {
                self.st = *sym;
                return *res;
            }
        }

        unreachable!("uncoverd char: <{}> on {{{}}}", ch, sym2str(self.st));
    }
}

#[macro_export]
macro_rules! declare_st {
    ( $($name:ident),* ) => {
        use $crate::concat_idents2;

        $(
            concat_idents2!(state_name = $name, _ST {
                const state_name: &'static str = stringify!($name);
            });
        )*
    };
}


#[macro_export]
macro_rules! lexdfamap {
    ( $($cur_st:expr =>
        {
            $( $matcher:ident | $nxt_st:expr, $flag:literal )*
        }
      ),*
    ) => {
        {
            use std::collections::HashMap;
            use $crate::FnCharMatcher;
            use $crate::concat_idents2;
            use $crate::str2sym;

            let mut _map = HashMap::new();

            $(
                let mut trans_vec = Vec::new();

                $(
                    let nxt_st = str2sym($nxt_st);

                    trans_vec.push((
                        concat_idents2!(matcher_name = $matcher, _m {
                            matcher_name as FnCharMatcher
                        }),
                        (nxt_st, $flag)
                    ));
                )*

                let cur_st = str2sym($cur_st);

                _map.insert(cur_st, trans_vec);
            )*

            _map
        }
    }
}


pub struct TokenRecognizer {
    pub lookhead: usize,
    pub pat_items: Vec<(Regex, Symbol)>,
}

impl TokenRecognizer {
    pub fn recognize(&self, source: &str, span: Span) -> Token {
        let end = min(span.end, span.from + self.lookhead);

        for (pat, name) in self.pat_items.iter() {
            if pat.is_match(&source[..end]) {
                return Token {
                    name: *name,
                    value: str2sym(&source[span.from..span.end]),
                    span,
                };
            }
        }

        unreachable!("Unreconized Raw Token: {}", &source[span.from..span.end])
    }
}


#[macro_export]
macro_rules! token_recognizer {
    ( $lookahead:literal | $($token_name:ident => $patstr:literal),* | ) => {
        {
            use $crate::Regex;
            use $crate::str2sym;
            use $crate::TokenRecognizer;

            let mut pat_items = vec![];

            $(
                let mut patstr = $patstr.to_owned();

                if patstr.starts_with("^") {
                    patstr.insert(0, '^')
                }

                pat_items.push((
                    Regex::new(&patstr).unwrap(),
                    str2sym(stringify!($token_name))
                ));
            )*

            TokenRecognizer {
                lookhead: $lookahead,
                pat_items
            }
        }
    }
}



pub fn tokenize2(
    srcfile: &SrcFileInfo,
    dfamap: &LexDFAMap,
    reconizer: &TokenRecognizer,
) -> TokenizeResult {
    let mut tokens = vec![];

    let mut dfa = LexDFA::new(dfamap);
    let mut bytes_pos = 0;
    let mut cache = String::new();

    for c in srcfile.srcstr.chars() {
        if dfa.forward(c) {
            // REACH TOKEN END
            // recognize token
            let span = Span {
                from: bytes_pos,
                end: bytes_pos + cache.len(),
            };
            bytes_pos += span.len();

            tokens.push(reconizer.recognize(&srcfile.srcstr, span));

            cache.clear();
        }

        cache.push(c);
    }

    Ok(tokens)
}



#[cfg(test)]
mod tests {

    #[test]
    fn test_error_info() {
        println!("aaaa\n^^^^^")
    }
}
