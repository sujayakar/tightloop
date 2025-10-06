use ratatui::text::{
    Line,
    Span,
    Text,
};
use syntect::{
    easy::HighlightLines,
    highlighting::ThemeSet,
    parsing::SyntaxSet,
    util::LinesWithEndings,
};
use syntect_tui::into_span;
use tracing::debug;

pub struct SyntaxHighlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
}

impl SyntaxHighlighter {
    pub fn new() -> Self {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();

        Self { syntax_set, theme_set }
    }

    pub fn highlight_code<'a>(&self, code: &'a str, language: Option<&str>) -> Text<'a> {
        debug!(
            "highlight_code called with language: {:?}, code length: {}",
            language,
            code.len()
        );
        debug!("Code snippet:\n{}", code);

        // Choose a theme (you can make this configurable)
        let theme = &self.theme_set.themes["base16-ocean.dark"];
        debug!("Using theme: {:?}", theme.name);

        // Find syntax by language token (tries extension first, then case-insensitive name)
        let syntax = language
            .and_then(|lang| {
                debug!("Looking for syntax by token: {}", lang);
                if let Some(syntax) = self.syntax_set.find_syntax_by_token(lang) {
                    return Some(syntax);
                }
                let path = std::path::Path::new(lang);
                if let Some(ext) = path.extension() {
                    if let Some(ext) = ext.to_str() {
                        debug!("Looking for syntax by extension: {}", ext);
                        if let Some(syntax) = self.syntax_set.find_syntax_by_extension(ext) {
                            return Some(syntax);
                        }
                    }
                }
                None
            })
            .unwrap_or_else(|| {
                debug!("No syntax found, defaulting to Plain Text");
                self.syntax_set.find_syntax_plain_text()
            });

        debug!(
            "Chosen syntax: {} (extensions: {:?})",
            syntax.name, syntax.file_extensions
        );

        let mut highlighter = HighlightLines::new(syntax, theme);

        let lines: Vec<Line<'a>> = LinesWithEndings::from(code)
            .map(|line| {
                let ranges = highlighter.highlight_line(line, &self.syntax_set).unwrap();
                debug!("Highlighted line with {} ranges", ranges.len());
                let spans: Vec<Span<'a>> = ranges
                    .into_iter()
                    .map(|(style, text)| {
                        let span = into_span((style, text)).unwrap();
                        debug!("Created span: '{}' with style: {:?}", span.content, span.style);
                        span
                    })
                    .collect();
                Line::from(spans)
            })
            .collect();

        debug!("Highlighted {} lines of code", lines.len());
        Text::from(lines)
    }
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}
