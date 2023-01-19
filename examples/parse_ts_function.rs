use indoc::indoc;
use jutus::program::Error;
use jutus::*;
use std::path::Path;

fn main() -> Result<(), Error> {
    let code = indoc! {r#"
    // adds two numbers
    // function add(a: number, b: number): number {
    //     return a + b;
    // }
    
    function identity(v: number): number {
        return v;
    }
    
    // not a real validator (one of: "spend", "cert", "mint", "withdrawal").
    function spend(sum: number, a: number, b: number): boolean {
        return sum == (a + b);
    }
  "#};

    js_compiler::parser_main_helper(code, Path::new(file!()))
}
