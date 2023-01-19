use indoc::indoc;
use jutus::program::Error;
use jutus::*;
use std::path::Path;

fn main() -> Result<(), Error> {
    let code = indoc! {r#"
    function test() {
        let b = 123;
        let sum = 1 + b;
        b + sum;
    }
  "#};

    js_compiler::parser_main_helper(code, Path::new(file!()))
}
