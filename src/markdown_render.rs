// Based off https://github.com/openai/codex/blob/main/codex-rs/tui/src/markdown_render.rs
//                                  Apache License
//                            Version 2.0, January 2004
//                         http://www.apache.org/licenses/
//
// TERMS AND CONDITIONS FOR USE, REPRODUCTION, AND DISTRIBUTION
//
// 1.  Definitions.
//
//     "License" shall mean the terms and conditions for use, reproduction,
//     and distribution as defined by Sections 1 through 9 of this document.
//
//     "Licensor" shall mean the copyright owner or entity authorized by
//     the copyright owner that is granting the License.
//
//     "Legal Entity" shall mean the union of the acting entity and all
//     other entities that control, are controlled by, or are under common
//     control with that entity. For the purposes of this definition,
//     "control" means (i) the power, direct or indirect, to cause the
//     direction or management of such entity, whether by contract or
//     otherwise, or (ii) ownership of fifty percent (50%) or more of the
//     outstanding shares, or (iii) beneficial ownership of such entity.
//
//     "You" (or "Your") shall mean an individual or Legal Entity
//     exercising permissions granted by this License.
//
//     "Source" form shall mean the preferred form for making modifications,
//     including but not limited to software source code, documentation
//     source, and configuration files.
//
//     "Object" form shall mean any form resulting from mechanical
//     transformation or translation of a Source form, including but
//     not limited to compiled object code, generated documentation,
//     and conversions to other media types.
//
//     "Work" shall mean the work of authorship, whether in Source or
//     Object form, made available under the License, as indicated by a
//     copyright notice that is included in or attached to the work
//     (an example is provided in the Appendix below).
//
//     "Derivative Works" shall mean any work, whether in Source or Object
//     form, that is based on (or derived from) the Work and for which the
//     editorial revisions, annotations, elaborations, or other modifications
//     represent, as a whole, an original work of authorship. For the purposes
//     of this License, Derivative Works shall not include works that remain
//     separable from, or merely link (or bind by name) to the interfaces of,
//     the Work and Derivative Works thereof.
//
//     "Contribution" shall mean any work of authorship, including
//     the original version of the Work and any modifications or additions
//     to that Work or Derivative Works thereof, that is intentionally
//     submitted to Licensor for inclusion in the Work by the copyright owner
//     or by an individual or Legal Entity authorized to submit on behalf of
//     the copyright owner. For the purposes of this definition, "submitted"
//     means any form of electronic, verbal, or written communication sent
//     to the Licensor or its representatives, including but not limited to
//     communication on electronic mailing lists, source code control systems,
//     and issue tracking systems that are managed by, or on behalf of, the
//     Licensor for the purpose of discussing and improving the Work, but
//     excluding communication that is conspicuously marked or otherwise
//     designated in writing by the copyright owner as "Not a Contribution."
//
//     "Contributor" shall mean Licensor and any individual or Legal Entity
//     on behalf of whom a Contribution has been received by Licensor and
//     subsequently incorporated within the Work.
//
// 2.  Grant of Copyright License. Subject to the terms and conditions of
//     this License, each Contributor hereby grants to You a perpetual,
//     worldwide, non-exclusive, no-charge, royalty-free, irrevocable
//     copyright license to reproduce, prepare Derivative Works of,
//     publicly display, publicly perform, sublicense, and distribute the
//     Work and such Derivative Works in Source or Object form.
//
// 3.  Grant of Patent License. Subject to the terms and conditions of
//     this License, each Contributor hereby grants to You a perpetual,
//     worldwide, non-exclusive, no-charge, royalty-free, irrevocable
//     (except as stated in this section) patent license to make, have made,
//     use, offer to sell, sell, import, and otherwise transfer the Work,
//     where such license applies only to those patent claims licensable
//     by such Contributor that are necessarily infringed by their
//     Contribution(s) alone or by combination of their Contribution(s)
//     with the Work to which such Contribution(s) was submitted. If You
//     institute patent litigation against any entity (including a
//     cross-claim or counterclaim in a lawsuit) alleging that the Work
//     or a Contribution incorporated within the Work constitutes direct
//     or contributory patent infringement, then any patent licenses
//     granted to You under this License for that Work shall terminate
//     as of the date such litigation is filed.
//
// 4.  Redistribution. You may reproduce and distribute copies of the
//     Work or Derivative Works thereof in any medium, with or without
//     modifications, and in Source or Object form, provided that You
//     meet the following conditions:
//
//     (a) You must give any other recipients of the Work or
//     Derivative Works a copy of this License; and
//
//     (b) You must cause any modified files to carry prominent notices
//     stating that You changed the files; and
//
//     (c) You must retain, in the Source form of any Derivative Works
//     that You distribute, all copyright, patent, trademark, and
//     attribution notices from the Source form of the Work,
//     excluding those notices that do not pertain to any part of
//     the Derivative Works; and
//
//     (d) If the Work includes a "NOTICE" text file as part of its
//     distribution, then any Derivative Works that You distribute must
//     include a readable copy of the attribution notices contained
//     within such NOTICE file, excluding those notices that do not
//     pertain to any part of the Derivative Works, in at least one
//     of the following places: within a NOTICE text file distributed
//     as part of the Derivative Works; within the Source form or
//     documentation, if provided along with the Derivative Works; or,
//     within a display generated by the Derivative Works, if and
//     wherever such third-party notices normally appear. The contents
//     of the NOTICE file are for informational purposes only and
//     do not modify the License. You may add Your own attribution
//     notices within Derivative Works that You distribute, alongside
//     or as an addendum to the NOTICE text from the Work, provided
//     that such additional attribution notices cannot be construed
//     as modifying the License.
//
//     You may add Your own copyright statement to Your modifications and
//     may provide additional or different license terms and conditions
//     for use, reproduction, or distribution of Your modifications, or
//     for any such Derivative Works as a whole, provided Your use,
//     reproduction, and distribution of the Work otherwise complies with
//     the conditions stated in this License.
//
// 5.  Submission of Contributions. Unless You explicitly state otherwise,
//     any Contribution intentionally submitted for inclusion in the Work
//     by You to the Licensor shall be under the terms and conditions of
//     this License, without any additional terms or conditions.
//     Notwithstanding the above, nothing herein shall supersede or modify
//     the terms of any separate license agreement you may have executed
//     with Licensor regarding such Contributions.
//
// 6.  Trademarks. This License does not grant permission to use the trade
//     names, trademarks, service marks, or product names of the Licensor,
//     except as required for reasonable and customary use in describing the
//     origin of the Work and reproducing the content of the NOTICE file.
//
// 7.  Disclaimer of Warranty. Unless required by applicable law or
//     agreed to in writing, Licensor provides the Work (and each
//     Contributor provides its Contributions) on an "AS IS" BASIS,
//     WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
//     implied, including, without limitation, any warranties or conditions
//     of TITLE, NON-INFRINGEMENT, MERCHANTABILITY, or FITNESS FOR A
//     PARTICULAR PURPOSE. You are solely responsible for determining the
//     appropriateness of using or redistributing the Work and assume any
//     risks associated with Your exercise of permissions under this License.
//
// 8.  Limitation of Liability. In no event and under no legal theory,
//     whether in tort (including negligence), contract, or otherwise,
//     unless required by applicable law (such as deliberate and grossly
//     negligent acts) or agreed to in writing, shall any Contributor be
//     liable to You for damages, including any direct, indirect, special,
//     incidental, or consequential damages of any character arising as a
//     result of this License or out of the use or inability to use the
//     Work (including but not limited to damages for loss of goodwill,
//     work stoppage, computer failure or malfunction, or any and all
//     other commercial damages or losses), even if such Contributor
//     has been advised of the possibility of such damages.
//
// 9.  Accepting Warranty or Additional Liability. While redistributing
//     the Work or Derivative Works thereof, You may choose to offer,
//     and charge a fee for, acceptance of support, warranty, indemnity,
//     or other liability obligations and/or rights consistent with this
//     License. However, in accepting such obligations, You may act only
//     on Your own behalf and on Your sole responsibility, not on behalf
//     of any other Contributor, and only if You agree to indemnify,
//     defend, and hold each Contributor harmless for any liability
//     incurred by, or claims asserted against, such Contributor by reason
//     of your accepting any such warranty or additional liability.
//
// END OF TERMS AND CONDITIONS
//
// APPENDIX: How to apply the Apache License to your work.
//
//       To apply the Apache License to your work, attach the following
//       boilerplate notice, with the fields enclosed by brackets "[]"
//       replaced with your own identifying information. (Don't include
//       the brackets!)  The text should be enclosed in the appropriate
//       comment syntax for the file format. We also recommend that a
//       file or class name and description of purpose be included on the
//       same "printed page" as the copyright notice for easier
//       identification within third-party archives.
//
// Copyright 2025 OpenAI
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//        http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.


use pulldown_cmark::{
    CodeBlockKind,
    CowStr,
    Event,
    HeadingLevel,
    Options,
    Parser,
    Tag,
    TagEnd,
};
use ratatui::{
    style::{
        Style,
        Stylize,
    },
    text::{
        Line,
        Span,
        Text,
    },
};
use tracing::debug;

use crate::syntax_highlight::SyntaxHighlighter;

// use crate::citation_regex::CITATION_REGEX;

#[derive(Clone, Debug)]
struct IndentContext {
    prefix: Vec<Span<'static>>,
    marker: Option<Vec<Span<'static>>>,
    is_list: bool,
}

impl IndentContext {
    fn new(prefix: Vec<Span<'static>>, marker: Option<Vec<Span<'static>>>, is_list: bool) -> Self {
        Self {
            prefix,
            marker,
            is_list,
        }
    }
}

pub(crate) fn render_markdown_text(input: &str) -> Text<'static> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(input, options);
    let mut w = Writer::new(parser, None, None);
    w.run();
    w.text
}

struct Writer<'a, I>
where I: Iterator<Item = Event<'a>>
{
    iter: I,
    text: Text<'static>,
    inline_styles: Vec<Style>,
    indent_stack: Vec<IndentContext>,
    list_indices: Vec<Option<u64>>,
    link: Option<String>,
    needs_newline: bool,
    pending_marker_line: bool,
    in_paragraph: bool,
    #[allow(unused)]
    scheme: Option<String>,
    #[allow(unused)]
    cwd: Option<std::path::PathBuf>,
    in_code_block: bool,
    code_block_lang: Option<String>,
    code_block_content: String,
    syntax_highlighter: SyntaxHighlighter,
}

impl<'a, I> Writer<'a, I>
where I: Iterator<Item = Event<'a>>
{
    fn new(iter: I, scheme: Option<String>, cwd: Option<std::path::PathBuf>) -> Self {
        Self {
            iter,
            text: Text::default(),
            inline_styles: Vec::new(),
            indent_stack: Vec::new(),
            list_indices: Vec::new(),
            link: None,
            needs_newline: false,
            pending_marker_line: false,
            in_paragraph: false,
            scheme,
            cwd,
            in_code_block: false,
            code_block_lang: None,
            code_block_content: String::new(),
            syntax_highlighter: SyntaxHighlighter::new(),
        }
    }

    fn run(&mut self) {
        while let Some(ev) = self.iter.next() {
            self.handle_event(ev);
        }
    }

    fn handle_event(&mut self, event: Event<'a>) {
        match event {
            Event::Start(tag) => self.start_tag(tag),
            Event::End(tag) => self.end_tag(tag),
            Event::Text(text) => self.text(text),
            Event::Code(code) => self.code(code),
            Event::SoftBreak => self.soft_break(),
            Event::HardBreak => self.hard_break(),
            Event::Rule => {
                if !self.text.lines.is_empty() {
                    self.push_blank_line();
                }
                self.push_line(Line::from("———"));
                self.needs_newline = true;
            }
            Event::Html(html) => self.html(html, false),
            Event::InlineHtml(html) => self.html(html, true),
            Event::FootnoteReference(_) => {}
            Event::TaskListMarker(_) => {}
            _ => {}
        }
    }

    fn start_tag(&mut self, tag: Tag<'a>) {
        match tag {
            Tag::Paragraph => self.start_paragraph(),
            Tag::Heading { level, .. } => self.start_heading(level),
            Tag::BlockQuote(..) => self.start_blockquote(),
            Tag::CodeBlock(kind) => {
                let indent = match kind {
                    CodeBlockKind::Fenced(_) => None,
                    CodeBlockKind::Indented => Some(Span::from(" ".repeat(4))),
                };
                let lang = match kind {
                    CodeBlockKind::Fenced(lang) => Some(lang.to_string()),
                    CodeBlockKind::Indented => None,
                };
                self.start_codeblock(lang, indent)
            }
            Tag::List(start) => self.start_list(start),
            Tag::Item => self.start_item(),
            Tag::Emphasis => self.push_inline_style(Style::new().italic()),
            Tag::Strong => self.push_inline_style(Style::new().bold()),
            Tag::Strikethrough => self.push_inline_style(Style::new().crossed_out()),
            Tag::Link { dest_url, .. } => self.push_link(dest_url.to_string()),
            Tag::HtmlBlock
            | Tag::FootnoteDefinition(_)
            | Tag::Table(_)
            | Tag::TableHead
            | Tag::TableRow
            | Tag::TableCell
            | Tag::Image { .. }
            | Tag::MetadataBlock(_)
            | _ => {}
        }
    }

    fn end_tag(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Paragraph => self.end_paragraph(),
            TagEnd::Heading(_) => self.end_heading(),
            TagEnd::BlockQuote(..) => self.end_blockquote(),
            TagEnd::CodeBlock => self.end_codeblock(),
            TagEnd::List(_) => self.end_list(),
            TagEnd::Item => {
                self.indent_stack.pop();
                self.pending_marker_line = false;
            }
            TagEnd::Emphasis | TagEnd::Strong | TagEnd::Strikethrough => self.pop_inline_style(),
            TagEnd::Link => self.pop_link(),
            TagEnd::HtmlBlock
            | TagEnd::FootnoteDefinition
            | TagEnd::Table
            | TagEnd::TableHead
            | TagEnd::TableRow
            | TagEnd::TableCell
            | TagEnd::Image
            | TagEnd::MetadataBlock(_)
            | _ => {}
        }
    }

    fn start_paragraph(&mut self) {
        if self.needs_newline {
            self.push_blank_line();
        }
        self.push_line(Line::default());
        self.needs_newline = false;
        self.in_paragraph = true;
    }

    fn end_paragraph(&mut self) {
        self.needs_newline = true;
        self.in_paragraph = false;
        self.pending_marker_line = false;
    }

    fn start_heading(&mut self, level: HeadingLevel) {
        if self.needs_newline {
            self.push_line(Line::default());
            self.needs_newline = false;
        }
        let heading_style = match level {
            HeadingLevel::H1 => Style::new().bold().underlined(),
            HeadingLevel::H2 => Style::new().bold(),
            HeadingLevel::H3 => Style::new().bold().italic(),
            HeadingLevel::H4 => Style::new().italic(),
            HeadingLevel::H5 => Style::new().italic(),
            HeadingLevel::H6 => Style::new().italic(),
        };
        let content = format!("{} ", "#".repeat(level as usize));
        self.push_line(Line::from(vec![Span::styled(content, heading_style)]));
        self.push_inline_style(heading_style);
        self.needs_newline = false;
    }

    fn end_heading(&mut self) {
        self.needs_newline = true;
        self.pop_inline_style();
    }

    fn start_blockquote(&mut self) {
        if self.needs_newline {
            self.push_blank_line();
            self.needs_newline = false;
        }
        self.indent_stack
            .push(IndentContext::new(vec![Span::from("> ")], None, false));
    }

    fn end_blockquote(&mut self) {
        self.indent_stack.pop();
        self.needs_newline = true;
    }

    fn text(&mut self, text: CowStr<'a>) {
        if self.pending_marker_line {
            self.push_line(Line::default());
        }
        self.pending_marker_line = false;

        if self.in_code_block {
            // Collect code block content for syntax highlighting
            debug!("Adding to code block content: {}", text);
            self.code_block_content.push_str(&text);
            return;
        }

        if self.in_code_block
            && !self.needs_newline
            && self
                .text
                .lines
                .last()
                .map(|line| !line.spans.is_empty())
                .unwrap_or(false)
        {
            self.push_line(Line::default());
        }
        for (i, line) in text.lines().enumerate() {
            if self.needs_newline {
                self.push_line(Line::default());
                self.needs_newline = false;
            }
            if i > 0 {
                self.push_line(Line::default());
            }
            let content = line.to_string();
            // if !self.in_code_block
            //     && let (Some(scheme), Some(cwd)) = (&self.scheme, &self.cwd)
            // {
            //     let cow = rewrite_file_citations_with_scheme(&content, Some(scheme.as_str()), cwd);
            //     if let std::borrow::Cow::Owned(s) = cow {
            //         content = s;
            //     }
            // }
            let span = Span::styled(content, self.inline_styles.last().copied().unwrap_or_default());
            self.push_span(span);
        }
        self.needs_newline = false;
    }

    fn code(&mut self, code: CowStr<'a>) {
        if self.pending_marker_line {
            self.push_line(Line::default());
        }
        self.pending_marker_line = false;
        let span = Span::from(code.into_string()).dim();
        self.push_span(span);
    }

    fn html(&mut self, html: CowStr<'a>, inline: bool) {
        self.pending_marker_line = false;
        for (i, line) in html.lines().enumerate() {
            if self.needs_newline {
                self.push_line(Line::default());
                self.needs_newline = false;
            }
            if i > 0 {
                self.push_line(Line::default());
            }
            let style = self.inline_styles.last().copied().unwrap_or_default();
            self.push_span(Span::styled(line.to_string(), style));
        }
        self.needs_newline = !inline;
    }

    fn hard_break(&mut self) {
        self.push_line(Line::default());
    }

    fn soft_break(&mut self) {
        self.push_line(Line::default());
    }

    fn start_list(&mut self, index: Option<u64>) {
        if self.list_indices.is_empty() && self.needs_newline {
            self.push_line(Line::default());
        }
        self.list_indices.push(index);
    }

    fn end_list(&mut self) {
        self.list_indices.pop();
        self.needs_newline = true;
    }

    fn start_item(&mut self) {
        self.pending_marker_line = true;
        let depth = self.list_indices.len();
        let is_ordered = self.list_indices.last().map(Option::is_some).unwrap_or(false);
        let width = depth * 4 - 3;
        let marker = if let Some(last_index) = self.list_indices.last_mut() {
            match last_index {
                None => Some(vec![Span::from(" ".repeat(width - 1) + "- ")]),
                Some(index) => {
                    *index += 1;
                    Some(vec![format!("{:width$}. ", *index - 1).light_blue()])
                }
            }
        } else {
            None
        };
        let indent_prefix = if depth == 0 {
            Vec::new()
        } else {
            let indent_len = if is_ordered { width + 2 } else { width + 1 };
            vec![Span::from(" ".repeat(indent_len))]
        };
        self.indent_stack.push(IndentContext::new(indent_prefix, marker, true));
        self.needs_newline = false;
    }

    fn start_codeblock(&mut self, lang: Option<String>, indent: Option<Span<'static>>) {
        debug!("Starting code block with language: {:?}", lang);
        if !self.text.lines.is_empty() {
            self.push_blank_line();
        }
        self.in_code_block = true;
        self.code_block_lang = lang;
        self.code_block_content.clear();
        self.indent_stack
            .push(IndentContext::new(vec![indent.unwrap_or_default()], None, false));
        self.needs_newline = true;
    }

    fn end_codeblock(&mut self) {
        debug!(
            "Ending code block. Language: {:?}, Content length: {}",
            self.code_block_lang,
            self.code_block_content.len()
        );
        debug!("Code block content:\n{}", self.code_block_content);

        // Apply syntax highlighting to the collected code block content
        let highlighted_text = self
            .syntax_highlighter
            .highlight_code(&self.code_block_content, self.code_block_lang.as_deref());

        debug!("Highlighted text has {} lines", highlighted_text.lines.len());

        // Convert the highlighted text to static spans by cloning the content
        let static_lines: Vec<Line<'static>> = highlighted_text
            .lines
            .into_iter()
            .map(|line| {
                let static_spans: Vec<Span<'static>> = line
                    .spans
                    .into_iter()
                    .map(|span| {
                        debug!("Converting span: '{}' with style: {:?}", span.content, span.style);
                        Span::styled(span.content.to_string(), span.style)
                    })
                    .collect();
                Line::from(static_spans)
            })
            .collect();

        debug!("Converted to {} static lines", static_lines.len());

        // Add the highlighted lines to our text
        for line in static_lines {
            self.push_line(line);
        }

        self.needs_newline = true;
        self.in_code_block = false;
        self.code_block_lang = None;
        self.code_block_content.clear();
        self.indent_stack.pop();
    }

    fn push_inline_style(&mut self, style: Style) {
        let current = self.inline_styles.last().copied().unwrap_or_default();
        let merged = current.patch(style);
        self.inline_styles.push(merged);
    }

    fn pop_inline_style(&mut self) {
        self.inline_styles.pop();
    }

    fn push_link(&mut self, dest_url: String) {
        self.link = Some(dest_url);
    }

    fn pop_link(&mut self) {
        if let Some(link) = self.link.take() {
            self.push_span(" (".into());
            self.push_span(link.cyan().underlined());
            self.push_span(")".into());
        }
    }

    fn push_line(&mut self, line: Line<'static>) {
        let mut line = line;
        let was_pending = self.pending_marker_line;
        let mut spans = self.current_prefix_spans();
        spans.append(&mut line.spans);
        let blockquote_active = self
            .indent_stack
            .iter()
            .any(|ctx| ctx.prefix.iter().any(|s| s.content.contains('>')));
        let style = if blockquote_active {
            Style::new().green()
        } else {
            line.style
        };
        self.text.lines.push(Line::from_iter(spans).style(style));
        if was_pending {
            self.pending_marker_line = false;
        }
    }

    fn push_span(&mut self, span: Span<'static>) {
        if let Some(last) = self.text.lines.last_mut() {
            last.push_span(span);
        } else {
            self.push_line(Line::from(vec![span]));
        }
    }

    fn push_blank_line(&mut self) {
        if self.indent_stack.iter().all(|ctx| ctx.is_list) {
            self.text.lines.push(Line::default());
        } else {
            self.push_line(Line::default());
        }
    }

    fn current_prefix_spans(&self) -> Vec<Span<'static>> {
        let mut prefix: Vec<Span<'static>> = Vec::new();
        let last_marker_index = if self.pending_marker_line {
            self.indent_stack
                .iter()
                .enumerate()
                .rev()
                .find_map(|(i, ctx)| if ctx.marker.is_some() { Some(i) } else { None })
        } else {
            None
        };
        let last_list_index = self.indent_stack.iter().rposition(|ctx| ctx.is_list);

        for (i, ctx) in self.indent_stack.iter().enumerate() {
            if self.pending_marker_line {
                if Some(i) == last_marker_index
                    && let Some(marker) = &ctx.marker
                {
                    prefix.extend(marker.iter().cloned());
                    continue;
                }
                if ctx.is_list && last_marker_index.is_some_and(|idx| idx > i) {
                    continue;
                }
            } else if ctx.is_list && Some(i) != last_list_index {
                continue;
            }
            prefix.extend(ctx.prefix.iter().cloned());
        }

        prefix
    }
}

// pub(crate) fn rewrite_file_citations_with_scheme<'a>(
//     src: &'a str,
//     scheme_opt: Option<&str>,
//     cwd: &Path,
// ) -> Cow<'a, str> {
//     let scheme: &str = match scheme_opt {
//         Some(s) => s,
//         None => return Cow::Borrowed(src),
//     };

//     CITATION_REGEX.replace_all(src, |caps: &regex_lite::Captures<'_>| {
//         let file = &caps[1];
//         let start_line = &caps[2];

//         // Resolve the path against `cwd` when it is relative.
//         let absolute_path = {
//             let p = Path::new(file);
//             let absolute_path = if p.is_absolute() {
//                 path_clean::clean(p)
//             } else {
//                 path_clean::clean(cwd.join(p))
//             };
//             // VS Code expects forward slashes even on Windows because URIs use
//             // `/` as the path separator.
//             absolute_path.to_string_lossy().replace('\\', "/")
//         };

//         // Render as a normal markdown link so the downstream renderer emits
//         // the hyperlink escape sequence (when supported by the terminal).
//         //
//         // In practice, sometimes multiple citations for the same file, but with a
//         // different line number, are shown sequentially, so we:
//         // - include the line number in the label to disambiguate them
//         // - add a space after the link to make it easier to read
//         format!("[{file}:{start_line}]({scheme}://file{absolute_path}:{start_line}) ")
//     })
// }

// #[cfg(test)]
// mod markdown_render_tests {
//     include!("markdown_render_tests.rs");
// }

// #[cfg(test)]
// mod tests {
//     use pretty_assertions::assert_eq;

//     use super::*;

//     #[test]
//     fn citation_is_rewritten_with_absolute_path() {
//         let markdown = "See 【F:/src/main.rs†L42-L50】 for details.";
//         let cwd = Path::new("/workspace");
//         let result = rewrite_file_citations_with_scheme(markdown, Some("vscode"), cwd);

//         assert_eq!(
//             "See [/src/main.rs:42](vscode://file/src/main.rs:42)  for details.",
//             result
//         );
//     }

//     #[test]
//     fn citation_followed_by_space_so_they_do_not_run_together() {
//         let markdown = "References on lines 【F:src/foo.rs†L24】【F:src/foo.rs†L42】";
//         let cwd = Path::new("/home/user/project");
//         let result = rewrite_file_citations_with_citations(markdown, Some("vscode"), cwd);

//         assert_eq!(
//             "References on lines [src/foo.rs:24](vscode://file/home/user/project/src/foo.rs:24) \
//              [src/foo.rs:42](vscode://file/home/user/project/src/foo.rs:42) ",
//             result
//         );
//     }

//     #[test]
//     fn citation_unchanged_without_file_opener() {
//         let markdown = "Look at 【F:file.rs†L1】.";
//         let cwd = Path::new("/");
//         let unchanged = rewrite_file_citations_with_scheme(markdown, Some("vscode"), cwd);

//         // The helper itself always rewrites – this test validates behaviour of
//         // append_markdown when `file_opener` is None.
//         let rendered = render_markdown_text_with_citations(markdown, None, cwd);
//         // Convert lines back to string for comparison.
//         let rendered: String = rendered
//             .lines
//             .iter()
//             .flat_map(|l| l.spans.iter())
//             .map(|s| s.content.clone())
//             .collect::<Vec<_>>()
//             .join("");
//         assert_eq!(markdown, rendered);
//         // Ensure helper rewrites.
//         assert_ne!(markdown, unchanged);
//     }
// }
