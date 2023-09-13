use tangic_reporting::{Color, ColorGenerator, Label, Report, Source};
use tracing::{self, Level};
use tracing_subscriber::filter::LevelFilter;

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
                UnifiedError::Parsing(e) => {
                        let mut report = Report::build(
                                tangic_reporting::ReportKind::Error,
                                path,
                                e.span.start,
                        );
                        match e.error {
                                tangic_parse::ParseError::CharParse(e) => {
                                        report.set_message(e.to_string());
                                        report.add_label(
                                                Label::new((path, e.span))
                                                        .with_color(b)
                                                        .with_message("here"),
                                        );
                                }
                                tangic_parse::ParseError::Expected { expected, found } => {
                                        report.set_message(format!(
                                                "expected {expected}, found {found:?}"
                                        ));
                                        report.add_label(
                                                Label::new((path, e.span))
                                                        .with_color(b)
                                                        .with_message("here"),
                                        );
                                }
                                tangic_parse::ParseError::ExpectedEofFound(token) => {
                                        report.set_message(format!(
                                                "expected end of file, found {token:?}"
                                        ));
                                        report.add_label(
                                                Label::new((path, e.span.start..e.span.start))
                                                        .with_color(a)
                                                        .with_message("this token"),
                                        );
                                }
                                tangic_parse::ParseError::IntError(i) => {
                                        report.set_message(i);
                                        report.add_label(
                                                Label::new((path, e.span))
                                                        .with_color(b)
                                                        .with_message("here"),
                                        );
                                }
                                tangic_parse::ParseError::InvalidToken { expected, found } => {
                                        report.set_message(format!(
                                                "expected {expected:?}, found {found:?}"
                                        ));
                                        report.add_label(
                                                Label::new((path, e.span))
                                                        .with_color(b)
                                                        .with_message("here"),
                                        );
                                }
                                tangic_parse::ParseError::Other(reason) => {
                                        report.set_message(reason);
                                        report.add_label(
                                                Label::new((path, e.span))
                                                        .with_color(b)
                                                        .with_message("here"),
                                        );
                                }
                                tangic_parse::ParseError::UnexpectedEof => {
                                        report.set_message("unexpected end of file");
                                        report.add_label(
                                                Label::new((path, e.span))
                                                        .with_color(b)
                                                        .with_message("here"),
                                        );
                                }
                        };

                        report.finish().eprint((path, Source::from(src)))?
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
