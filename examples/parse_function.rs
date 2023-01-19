use indoc::indoc;
use jutus::program::Error;
use jutus::*;
use std::path::Path;

fn main() -> Result<(), Error> {
    let code = indoc! {r#"
    // adds two numbers
    function add(a, b) {
        let sum = a + b;
        return sum;
    }

    // subtracts two numbers
    function sub(a, b) { return a - b; }

    // the maximum of two numbers
    function max(a, b) {
        if(a >= b)
            return a;
        else
            return b;
    }
    
    function TheTrue()  { return true; }
    function TheFalse() { return false; }
  "#};

    js_compiler::parser_main_helper(code, Path::new(file!()))
}
