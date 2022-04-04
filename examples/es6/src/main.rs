use std::path::PathBuf;

use m6tokenizer::{Token, SrcFileInfo};
use maplit::hashset;

use es6::tokenize1;


fn main() {
    // let res = lit_regex_m(r#"/[|\\{}()[\]^$+*?.]/g"#, 0);
    // println!("res: {:?}", res);

    for i in 0..1 {
        // let path = PathBuf::from(format!("./examples/exp{}.js", i));
        // let srcfile = SrcFileInfo::new(path).unwrap();

        // // println!("{:#?}", sp_m(srcfile.get_srcstr(), SrcLoc { ln: 0, col: 0 }));

        // match tokenize1(&srcfile) {
        //     Ok(tokens) => {
        //         let tokens = trim_tokens(&tokens[..]);
        //         display_pure_tok(&tokens[..]);
        //     },
        //     Err(err) => println!("{}", err),
        // }
    }

}
