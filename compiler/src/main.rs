use miette::{GraphicalTheme, IntoDiagnostic, NamedSource, Result, ThemeCharacters, ThemeStyles, RgbColors};

fn main() -> Result<()> {
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
    let ast =
        tangic_parser::parse(input.clone(), NamedSource::new("small.tn", input))?;

    println!("{ast:#?}");

    Ok(())
}
