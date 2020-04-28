#![allow(dead_code)]

use super::encoders::winansi;
use crate::pdf::font::Font;
use regex::Regex;
use std::io::Write;

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
    pub text: String,
    pub tag: Tag,
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

    /// Get number of characters in text.
    pub fn get_length(&self) -> usize {
        self.text.chars().count()
    }

    /// Get width of text
    pub fn get_width(&self, font: &'static Font, font_size: f32) -> f32 {
        let text = format!("{} ", self.text); // add one space
        font.get_width(font_size, &text)
    }

    /// Get encoded text
    pub fn encoded_text(&self) -> Vec<u8> {
        TextSpan::encode_text(self.text.as_str())
    }

    /// Generates encoded text
    pub fn encode_text(text: &str) -> Vec<u8> {
        let text = format!("{} ", text); // add one space
        let encoded_text = winansi::encode(&text);
        let mut output: Vec<u8> = Vec::new();
        output.write_all(b"(").unwrap();
        output.write_all(&encoded_text).unwrap();
        output.write_all(b") Tj ").unwrap();
        output
    }

    /// Generates encoded spans
    pub fn encoded_spans(spans: &Vec<TextSpan>) -> Vec<u8> {
        let mut output: Vec<u8> = Vec::new();
        for span in spans {
            output.write_all(&span.encoded_text()).unwrap();
        }
        output
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
    use crate::pdf::models::Paragraph;
    use crate::pdf::styles::{HorizontalAlign, ParagraphStyle};
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
        let wrapped = p.wrap_to_width(300.0);
        println!("{:?}", wrapped);
        assert_eq!(wrapped.last().unwrap().last().unwrap().text, ". Ends here.");
    }
}
