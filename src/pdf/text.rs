#![allow(dead_code)]

use super::encoders::winansi;
use super::models::Paragraph;
use super::styles::HorizontalAlign;
use regex::Regex;
use std::io::Write;

pub struct Text {}

impl Text {
    pub fn get_text_line(
        input: &str,
        text_align: HorizontalAlign,
        width_offset: f32,
        indent: f32,
    ) -> Vec<u8> {
        let encoded_text = winansi::encode(input);
        let mut output: Vec<u8> = Vec::new();
        match text_align {
            // No intendation for centered or right aligned text.
            HorizontalAlign::Center => {
                write!(output, "{} 0 Td (", width_offset / 2.0).unwrap();
                output.write_all(&encoded_text).unwrap();
                output.write_all(b") Tj T* ").unwrap();
            }
            HorizontalAlign::Right => {
                write!(output, "{} 0 Td (", width_offset).unwrap();
                output.write_all(&encoded_text).unwrap();
                output.write_all(b") Tj T* ").unwrap();
            }
            _ => {
                if indent != 0.0 {
                    write!(output, "{} 0 Td (", indent).unwrap();
                } else {
                    output.write_all(b"(").unwrap();
                }
                output.write_all(&encoded_text).unwrap();
                output.write_all(b") Tj T* ").unwrap();
            }
        }
        output
    }

    pub fn get_bullet_text(input: &str) -> Vec<u8> {
        let encoded_text = winansi::encode(input);
        let mut output: Vec<u8> = Vec::new();
        output.write_all(b"(").unwrap();
        output.write_all(&encoded_text).unwrap();
        output.write_all(b") Tj ").unwrap();
        output
    }

    /// Returns lines of encoded text fitted to frame width as
    /// lines of encoded bytes (marked with postscript cmds)
    /// and pure string lines (eg. for dealing with page breaks).
    pub fn get_text_lines(
        paragraph: &Paragraph,
        text: &str,
        frame_width: f32,
    ) -> (Vec<Vec<u8>>, Vec<String>) {
        let mut encoded_lines: Vec<Vec<u8>> = Vec::new();
        let mut text_lines: Vec<String> = Vec::new();
        let font = &paragraph.font;
        let size = paragraph.font_size;
        let mut bullet_indent = paragraph.style.bullet_indent;
        let text_align = paragraph.style.align;
        let line_width = font.get_width(size, text);
        let mut previous_line_width = frame_width;
        // split words by any whitespace characters.
        let words: Vec<&str> = text.split_whitespace().collect();
        if line_width <= frame_width {
            // just adding one line of text (fits to width)
            // remove extra whitespace characters (such as linebreaks)
            let input_line = words.join(" ");
            let output = Text::get_text_line(
                &input_line,
                text_align,
                frame_width - line_width,
                bullet_indent,
            );
            encoded_lines.push(output);
            text_lines.push(input_line);
        } else {
            let mut line_strings = Vec::new();
            let mut next_line_word: Option<String> = None;
            for word in &words {
                if let Some(next_word) = next_line_word {
                    line_strings.push(next_word);
                    next_line_word = None;
                }
                line_strings.push((*word).to_string());
                if font.get_width(size, &line_strings.join(" ")) > frame_width {
                    // add last word that didn't fit to next line
                    next_line_word = Some((*word).to_string());
                    line_strings.pop();
                    let output_line = line_strings.join(" ");
                    let line_width = font.get_width(size, &output_line);
                    let output = Text::get_text_line(
                        &output_line,
                        text_align,
                        previous_line_width - line_width,
                        bullet_indent,
                    );
                    bullet_indent = 0.0; // reset bullet indent
                    previous_line_width = line_width;
                    encoded_lines.push(output);
                    text_lines.push(output_line);
                    line_strings = Vec::new();
                }
            }
            // output last line
            if let Some(next_word) = next_line_word {
                line_strings.push(next_word);
            }
            let output_line = line_strings.join(" ");
            let line_width = font.get_width(size, &output_line);
            let output = Text::get_text_line(
                &output_line,
                text_align,
                previous_line_width - line_width,
                bullet_indent,
            );
            encoded_lines.push(output);
            text_lines.push(output_line);
        }
        (encoded_lines, text_lines)
    }
    /// Get max line width.
    pub fn get_text_width(paragraph: &Paragraph, text: &str, frame_width: f32) -> f32 {
        let font = &paragraph.font;
        let size = paragraph.font_size;
        let width = font.get_width(size, text);
        if width <= frame_width {
            width
        } else {
            let mut max_width = 0.0;
            // split words by any whitespace characters.
            let words: Vec<&str> = text.split_whitespace().collect();
            let mut line_strings = Vec::new();
            let mut next_line_word: Option<String> = None;
            for word in &words {
                if let Some(next_word) = next_line_word {
                    line_strings.push(next_word);
                    next_line_word = None;
                }
                line_strings.push((*word).to_string());
                let curr_width = font.get_width(size, &line_strings.join(" "));
                if curr_width > frame_width {
                    // add last word that didn't fit to next line
                    next_line_word = Some((*word).to_string());
                    line_strings = Vec::new();
                } else if max_width < curr_width {
                    max_width = curr_width;
                }
            }
            max_width
        }
    }
}

#[derive(Debug, Clone)]
pub enum Tag {
    Span,
    Link { url: String },
    Bold,
}

/// TextSpan contains fragment of paragraph text,
/// that may contain some attributes.
#[derive(Debug, Clone)]
pub struct TextSpan {
    text: String,
    tag: Tag,
}

impl TextSpan {
    pub fn new(text: String, tag: Tag) -> TextSpan {
        TextSpan { text, tag }
    }
    /// Generate all spans for given text.
    pub fn extract_spans(p_text: &str) -> Vec<TextSpan> {
        lazy_static! {
            static ref RE: Regex =
                Regex::new(r"<a[\s]+href='(?P<url>[^']+)'[^>]*?>(?P<link>.*?)</a>").unwrap();
        }
        let mut text_parts: Vec<TextSpan> = Vec::new();
        let mut current_index = 0;
        for capture in RE.captures_iter(p_text) {
            if let Some(m) = capture.get(0) {
                let start_index = m.start();
                let end_index = m.end();
                if start_index > current_index {
                    let text: &str = &p_text[current_index..start_index];
                    let span = TextSpan::new(String::from(text), Tag::Span);
                    text_parts.push(span);
                }
                if let (Some(url), Some(link)) = (capture.name("url"), capture.name("link")) {
                    let span = TextSpan::new(
                        String::from(link.as_str()),
                        Tag::Link {
                            url: String::from(url.as_str()),
                        },
                    );
                    text_parts.push(span);
                }
                current_index = end_index;
            }
        }
        if current_index < p_text.len() - 1 {
            let text: &str = &p_text[current_index..];
            let span = TextSpan::new(String::from(text), Tag::Span);
            text_parts.push(span);
        }
        text_parts
    }
    /// Generate wrapped text spans, line may contain multiple spans
    /// and a span may split to next lines. This is NOT optimal, but it works..
    pub fn wrap_to_width(
        spans: &Vec<TextSpan>,
        paragraph: &Paragraph,
        available_width: f32,
    ) -> Vec<Vec<TextSpan>> {
        let font = paragraph.font;
        let size = paragraph.font_size;
        // contain lines of lines of spans
        let mut wrapped: Vec<Vec<TextSpan>> = Vec::new();
        // contains line of spans
        let mut line_spans: Vec<TextSpan> = Vec::new();
        // contains words per line
        let mut line_words: Vec<String> = Vec::new();
        for span in spans {
            let words: Vec<&str> = span.text.split_whitespace().collect();
            let mut next_word: Option<String> = None;
            let mut span_words: Vec<String> = Vec::new();
            for word in words {
                if let Some(_next_word) = next_word {
                    line_words.push(_next_word.clone());
                    span_words.push(_next_word);
                    next_word = None;
                }
                line_words.push(word.to_string());
                let current_width = font.get_width(size, &line_words.join(" "));
                if current_width > available_width {
                    next_word = Some(word.to_string());
                    let span_text = span_words.join(" ");
                    let text_span = TextSpan::new(span_text.to_string(), span.tag.clone());
                    line_spans.push(text_span);
                    wrapped.push(line_spans);
                    line_words = Vec::new();
                    line_spans = Vec::new();
                    span_words = Vec::new();
                } else {
                    span_words.push(word.to_string());
                }
            }
            if span_words.len() > 0 || next_word != None {
                if let Some(_next_word) = next_word {
                    span_words.push(_next_word);
                }
                let span_text = span_words.join(" ");
                let text_span = TextSpan::new(span_text.to_string(), span.tag.clone());
                line_spans.push(text_span);
            }
        }
        if line_spans.len() > 0 {
            wrapped.push(line_spans);
        }
        wrapped
    }
    /// Get number of characters in text.
    pub fn get_length(&self) -> usize {
        self.text.chars().count()
    }
}

pub fn extract_links(text: &str) -> String {
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r"<a[\s]+href='(?P<url>[^']+)'[^>]*?>(?P<link>.*?)</a>").unwrap();
    }
    RE.replace_all(text, "$link").into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pdf::styles::ParagraphStyle;
    use crate::pdf::units::Color;
    use lazy_static;

    #[test]
    fn test_link_removal() {
        lazy_static! {
            static ref RE: Regex =
                Regex::new(r"<a[\s]+href='(?P<url>[^']+)'[^>]*?>(?P<link>.*?)</a>").unwrap();
        }
        let output: String = RE
            .replace_all(
                "<a href='https://www.google.com'>A Link to Google</a>",
                "$link",
            )
            .into();
        assert_eq!(output.as_str(), "A Link to Google");
    }

    #[test]
    fn test_link_extraction() {
        let sample_text = "<a href='https://www.microsoft.com'>Microsoft Corporation</a>. Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
        <a href='https://www.google.com'>A Link to Google</a>. Aliquam maximus tincidunt nisl. <a href='https://www.yaloo.com'>A Link to Yahoo</a>. Ends here.";
        let text_parts = TextSpan::extract_spans(&sample_text);
        // println!("{:?}", text_parts);
        assert_eq!(
            text_parts[text_parts.len() - 1].text.as_str(),
            ". Ends here."
        );
    }

    #[test]
    fn test_wrap_to_width() {
        let sample_text = "<a href='https://www.microsoft.com'>Microsoft Corporation</a>. Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
        <a href='https://www.google.com'>A Link to Google</a>. Aliquam maximus tincidunt nisl. <a href='https://www.yaloo.com'>A Link to Yahoo</a>. Ends here.";
        let spans = TextSpan::extract_spans(&sample_text);
        let color = Color::new(0.0, 0.0, 0.0);
        let style: ParagraphStyle = ParagraphStyle::new(
            12.0,
            HorizontalAlign::Left,
            None,
            0.0,
            (0.0, 0.0, 0.0, 0.0),
            color,
        );
        let p: Paragraph = Paragraph::new(&sample_text, "helvetica", 12.0, style);
        let wrapped = TextSpan::wrap_to_width(&spans, &p, 300.0);
        println!("{:?}", wrapped);
        // let mut line_text: Vec<String> = Vec::new();
        // for line in wrapped {
        //     for elem in line {
        //         line_text.push(elem.text);
        //     }
        // }
        // println!("{:?}", line_text.join(" "));
        assert_eq!(wrapped.last().unwrap().last().unwrap().text, ". Ends here.");
    }
}
