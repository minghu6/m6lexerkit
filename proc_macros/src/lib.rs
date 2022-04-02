extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::{parse_macro_input, Ident, LitStr, Token};


////////////////////////////////////////////////////////////////////////////////
//// MakeCharMatcher

struct MakeCharMatcherRules {
    // ident, patstr
    rules: Vec<(Ident, LitStr)>,
}

impl Parse for MakeCharMatcherRules {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut rules = vec![];

        while !input.is_empty() {
            let name = input.parse()?;
            input.parse::<Token![=>]>()?;
            let patstr = input.parse::<LitStr>()?;

            // let patstr = LitStr::new(
            //     &("^$".to_string() + &patstr.value()),
            //     Span::call_site(),
            // );

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }

            rules.push((name, patstr));
        }

        Ok(Self { rules })
    }
}

#[proc_macro]
pub fn make_char_matcher_rules(input: TokenStream) -> TokenStream {
    let MakeCharMatcherRules { rules } =
        parse_macro_input!(input as MakeCharMatcherRules);

    let mut token_stream = quote! {
        use m6tokenizer::lazy_static::lazy_static;
        use m6tokenizer::RegexCharMatcher;
    };

    for (name, patstr) in rules {
        let matcher_fn_name = Ident::new(
            &format!("{}_m", name.to_string().to_lowercase()),
            Span::call_site(),
        );
        let matcher_reg_name = Ident::new(
            &format!("{}_REG", name.to_string().to_uppercase()),
            Span::call_site(),
        );
        token_stream.extend(quote! {
            pub fn #matcher_fn_name(c: &char) -> bool {
                lazy_static! {
                    static ref #matcher_reg_name: RegexCharMatcher = RegexCharMatcher::new(#patstr);
                }

                #matcher_reg_name.is_match(c)
            }
        })
    }

    TokenStream::from(token_stream)
}


////////////////////////////////////////////////////////////////////////////////
//// TokenMatcher

#[allow(unused)]
struct TokenMatcherRules {
    // ident, patstr
    rules: Vec<(Ident, Option<LitStr>)>,
}

impl Parse for TokenMatcherRules {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut rules = vec![];

        while !input.is_empty() {
            let name = input.parse()?;

            if input.peek(Token![=>]) {
                input.parse::<Token![=>]>()?;
                let patstr = input.parse::<LitStr>()?;
                rules.push((name, Some(patstr)))
            }
            else {
                rules.push((name, None))
            }

            if !input.is_empty() {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self { rules })
    }
}

#[proc_macro]
pub fn make_token_matcher_rules(input: TokenStream) -> TokenStream {
    let TokenMatcherRules { rules } =
        parse_macro_input!(input as TokenMatcherRules);

    let mut token_stream = quote! {
        use m6tokenizer::{
            Token,
            SrcLoc
        };
    };

    let mut matchers_ts = quote! {};

    for (name, patstr_opt) in rules {

        let matcher_fn_name = Ident::new(
            &format!("{}_m", name.to_string().to_lowercase()),
            Span::call_site(),
        );

        if let Some(patstr) = patstr_opt {
            let matcher_reg_name = Ident::new(
                &format!("{}_REG", name.to_string().to_uppercase()),
                Span::call_site(),
            );
            let adjust_patstr =
                LitStr::new(&format!("^({})", patstr.value()), Span::call_site());

            token_stream.extend(quote! {
                pub fn #matcher_fn_name(s: &str, loc: SrcLoc) -> Option<TokenMatchResult> {
                    m6tokenizer::lazy_static::lazy_static! {
                        static ref #matcher_reg_name: m6tokenizer::TokenMatcher
                            = m6tokenizer::TokenMatcher::new(#adjust_patstr, stringify!(#name));
                    }

                    #matcher_reg_name.fetch_tok(s, loc)
                }
            });
        }

        matchers_ts.extend(quote! { #matcher_fn_name as m6tokenizer::FnMatcher, });
    }

    token_stream.extend(quote! {
        m6tokenizer::lazy_static::lazy_static! {
            pub static ref MATCHERS: Vec<m6tokenizer::FnMatcher> = vec![#matchers_ts];
        }
    });

    TokenStream::from(token_stream)
}
