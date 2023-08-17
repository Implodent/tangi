use tangic_parse::WithSpan;

fn report(error: tangic_parse::ParseError, span: Range<usize>) {
        tangic_reporting::rll;
}

fn main() {
        let src = std::fs::read_to_string(std::env::args().skip(1).next().unwrap()).unwrap();
        match tangic_parse::Parser::new(&src).ask_file() {
                Ok(ast) => println!("happy path: {ast:#?}"),
                Err(WithSpan(error, span)) => {}
        }
}
