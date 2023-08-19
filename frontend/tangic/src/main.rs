use tangic_reporting::{Color, ColorGenerator, Label, Report, Source};

enum UnifiedError {
        Parsing(tangic_parse::Error),
}

fn report(
        error: UnifiedError,
        src: &str,
        path: &str,
        a: Color,
        b: Color,
) -> Result<(), std::io::Error> {
        match error {
                UnifiedError::Parsing(mut parsing) => {
                        for err in parsing.errors {
                                Report::build(
                                        tangic_reporting::ReportKind::Error,
                                        path,
                                        err.1.start,
                                )
                                .with_code(1)
                                .with_message(format!("{}", err.0))
                                .with_label(
                                        Label::new((path, err.1))
                                                .with_message("here")
                                                .with_color(a),
                                )
                                .finish()
                                .eprint((path, Source::from(src)))?
                        }
                }
        }
        Ok(())
}

fn main() {
        let path = std::env::args().skip(1).next().unwrap();
        let src = std::fs::read_to_string(&path).unwrap();
        match tangic_parse::parse(&src) {
                Ok(ast) => println!("{ast:#?}"),
                Err(errors) => {
                        let mut gen = ColorGenerator::new();
                        let a = gen.next();
                        let b = gen.next();
                        for error in errors {
                                report(UnifiedError::Parsing(error), &src, &path, a, b).unwrap();
                        }
                }
        }
}
