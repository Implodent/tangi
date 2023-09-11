use tangic_reporting::{Color, ColorGenerator, Label, Report, Source};
use tracing::{self, Level};
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

enum UnifiedError {
        Parsing(tangic_parse::Error),
}

fn report(
        error: UnifiedError,
        src: &str,
        path: &str,
        a: Color,
        _b: Color,
) -> Result<(), std::io::Error> {
        match error {
                UnifiedError::Parsing(e) => {
                        Report::build(tangic_reporting::ReportKind::Error, path, e.span.start)
                                .with_code(1)
                                .with_message(format!("{}", e))
                                .with_label(
                                        Label::new((path, e.span))
                                                .with_message("here")
                                                .with_color(a),
                                )
                                .finish()
                                .eprint((path, Source::from(src)))?
                }
        }
        Ok(())
}

fn main() {
        tracing_subscriber::fmt()
                .pretty()
                .with_max_level(LevelFilter::from_level(Level::TRACE))
                .with_file(true)
                .init();
        // unsafe { backtrace_on_stack_overflow::enable() };

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
