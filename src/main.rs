use solfaml_parser::parser::staff_parser;
use winnow::Parser;

fn main() {
    let source = "| d |

| d |
| d |
> ya 
| d |";
    let res = staff_parser.parse(source);
    println!("{res:?}")
}
