use miette::{GraphicalTheme, IntoDiagnostic, NamedSource, Result, ThemeCharacters, ThemeStyles, RgbColors};

use tracing::*;

fn main() -> Result<()> {
    tracing_subscriber::fmt().with_env_filter(tracing_subscriber::EnvFilter::from_env("TANGIC_LOG")).without_time().with_file(true).init();
    trace!("yeetus");

    miette::set_hook(Box::new(|_| {
        Box::new(
            miette::MietteHandlerOpts::new()
                .graphical_theme(GraphicalTheme {
                    styles: ThemeStyles::ansi(),
                    characters: ThemeCharacters::unicode(),
                })
                .with_cause_chain()
                .rgb_colors(RgbColors::Preferred)
                .build(),
        )
    }))?;

    let input = std::fs::read_to_string("small.tn").into_diagnostic()?;

    info!(?input, "Parsing\n");

    let (ast, errors) =
        tangic_parser::parse(input.clone(), NamedSource::new("small.tn", input))?;

    if errors.errors.is_empty() {
        println!("{ast:#?}");
    } else {
        eprintln!("{errors:?}");
    }

    Ok(())
}
