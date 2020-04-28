use std::io::Write;
use std::str;
use wasm_bindgen::prelude::*;

use super::font::Font;
use super::models::{Cell, Image, Paragraph, Path, Row, Spacer, Stationary, Table};
use super::objects::{LinkAnnotation, PDFDocument, PDFImage, PDFPage};
use super::styles::{HorizontalAlign, VerticalAlign};
use super::template::PageTemplate;
use super::units::{Color, Line, Point, Rect};
use crate::pdf::text::{Tag, TextSpan};

pub struct Canvas {
    output: Vec<u8>,
    pub cursor: (f32, f32),
    template: PageTemplate,
    doc: PDFDocument,
    images: Vec<PDFImage>,
    link_annotations: Vec<LinkAnnotation>,
}

impl Canvas {
    pub fn new(tpl: &PageTemplate) -> Canvas {
        let top_left = (tpl.get_frame().x, tpl.get_frame().y);
        let doc = PDFDocument::new();
        let output = Vec::new();
        let mut canvas = Canvas {
            output,
            cursor: top_left,
            template: tpl.clone(),
            doc,
            images: Vec::new(),
            link_annotations: Vec::new(),
        };
        canvas.write_preamble();
        canvas
    }
    pub fn _get_output(&self) -> Vec<u8> {
        self.output.clone()
    }
    /// Save the current graphics state to be restored later by restore_state.
    pub fn save_state(&mut self) {
        writeln!(self.output, "q").unwrap();
    }
    pub fn restore_state(&mut self) {
        writeln!(self.output, "Q").unwrap();
    }
    /// All canvas pages are initialized with preamble.
    pub fn write_preamble(&mut self) {
        writeln!(self.output, "1 0 0 1 0 0 cm  BT /F1 12 Tf 14.4 TL ET").unwrap();
        for element in self.template.stationary() {
            match element {
                Stationary::PageNumber {
                    font_size,
                    font,
                    x,
                    y,
                    color,
                } => {
                    let number = self.doc.page_number().to_string();
                    let point = Point { x, y };
                    self.draw_text_line(&number, font_size, &font, point, color);
                }
                Stationary::Text {
                    text,
                    font_size,
                    font,
                    x,
                    y,
                    color,
                } => {
                    let point = Point { x, y };
                    self.draw_text_line(&text, font_size, &font, point, color);
                }
            }
        }
    }
    fn draw_text_line(
        &mut self,
        text: &str,
        font_size: f32,
        font: &Font,
        point: Point,
        color: Color,
    ) {
        self.save_state();
        self.translate(point.x, point.y);
        self.save_state();
        self.set_fill_color(color.r, color.g, color.b);
        let mut out_text: Vec<u8> = Vec::new();
        let encoded_text = TextSpan::encode_text(text);
        out_text.extend(encoded_text);
        let mut stream = Vec::new();
        let leading = font_size;
        write!(
            stream,
            "BT 1 0 0 1 0 2 Tm /{} {} Tf {} TL ",
            font.get_ref(),
            font_size,
            leading
        )
        .unwrap();
        stream.write_all(&out_text).unwrap();
        writeln!(stream, " ET").unwrap();
        self.output.write_all(&stream).unwrap();
        self.restore_state();
        self.restore_state();
    }
    fn transform(&mut self, aa: &str, bb: &str, cc: &str, dd: &str, ee: &str, ff: &str) {
        writeln!(self.output, "{} {} {} {} {} {} cm", aa, bb, cc, dd, ee, ff).unwrap();
    }
    /// move the origin from the current (0,0) point to the (dx,dy) point
    /// (with respect to the current graphics state).
    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.transform("1", "0", "0", "1", &dx.to_string(), &dy.to_string());
    }
    pub fn set_fill_color(&mut self, r: f32, g: f32, b: f32) {
        writeln!(self.output, "{} {} {} rg", r, g, b).unwrap();
    }
    pub fn set_stroke_color(&mut self, r: f32, g: f32, b: f32) {
        writeln!(self.output, "{} {} {} RG", r, g, b).unwrap();
    }
    pub fn set_line_width(&mut self, width: f32) {
        writeln!(self.output, "{} w", width).unwrap();
    }
    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        self.save_state();
        self.set_fill_color(color.r, color.g, color.b);
        writeln!(
            self.output,
            "n {} {} {} {} re f*",
            rect.x, rect.y, rect.w, -rect.h
        )
        .unwrap();
        self.restore_state();
    }
    pub fn draw_line(&mut self, line: Line) {
        writeln!(
            self.output,
            "n {} {} m {} {} l S",
            line.x, line.y, line.x2, line.y2
        )
        .unwrap();
    }
    pub fn save_page(&mut self) {
        let mut page = PDFPage::new();
        page.set_contents(&self.output);
        page.set_images(&self.images);
        page.set_link_annotations(&self.link_annotations);
        self.doc.add_page(page);
        self.output = Vec::new();
        self.images = Vec::new();
        self.link_annotations = Vec::new();
        self.write_preamble();
        let top_left = (self.template.get_frame().x, self.template.get_frame().y);
        self.cursor = top_left;
    }
    pub fn set_cursor(&mut self, x: f32, y: f32) {
        self.cursor = (x, y);
    }
    pub fn draw_spacer(&mut self, spacer: &Spacer) -> Result<(), JsValue> {
        self.set_cursor(self.cursor.0 + spacer.width, self.cursor.1 - spacer.height);
        self.save_state();
        self.translate(self.cursor.0, self.cursor.1);
        self.restore_state();
        Ok(())
    }
    pub fn draw_path(&mut self, path: &Path, available_width: f32) -> Result<(), JsValue> {
        let pos_x = match path.style.horizontal_align {
            HorizontalAlign::Left => self.cursor.0,
            HorizontalAlign::Center => self.cursor.0 + (available_width - path.width) / 2.0,
            _ => self.cursor.0 + available_width - path.width,
        };
        self.save_state();
        self.translate(pos_x, self.cursor.1 - path.height);
        if let Some(stroke_color) = path.stroke_color {
            self.set_stroke_color(stroke_color.r, stroke_color.g, stroke_color.b);
        }
        if let Some(fill_color) = path.fill_color {
            self.set_fill_color(fill_color.r, fill_color.g, fill_color.b);
        }
        self.set_line_width(path.stroke_width);
        let mut init_point_drawn = false;
        for point in &path.points {
            if !init_point_drawn {
                writeln!(self.output, "n {} {} m", point.x, point.y).unwrap();
                init_point_drawn = true;
            } else {
                writeln!(self.output, "{} {} l", point.x, point.y).unwrap();
            }
        }
        writeln!(self.output, "h").unwrap(); // close path
        if path.fill_color.is_some() && path.stroke_color.is_some() && path.stroke_width > 0.0 {
            writeln!(self.output, "B").unwrap();
        } else if path.fill_color.is_some() {
            writeln!(self.output, "f").unwrap();
        } else if path.stroke_color.is_some() && path.stroke_width > 0.0 {
            writeln!(self.output, "S").unwrap();
        }
        self.restore_state();
        self.set_cursor(self.cursor.0, self.cursor.1 - path.height);
        Ok(())
    }
    fn draw_lines(&mut self, lines: Vec<Line>, table: &Table) {
        let style = &table.style;
        self.save_state();
        self.set_stroke_color(style.grid_color.r, style.grid_color.g, style.grid_color.b);
        self.set_line_width(style.grid_width);
        for line in lines {
            self.draw_line(line);
        }
        self.restore_state();
    }
    fn draw_table_row(
        &mut self,
        table: &Table,
        row: &Row,
        table_cursor: (f32, f32),
        is_first_row: bool,
        new_page: bool,
    ) -> Result<(), JsValue> {
        let frame_bottom = self.template.get_frame().y - self.template.get_frame().height;
        let horizontal_padding = table.style.padding_left + table.style.padding_right;
        let vertical_padding = table.style.padding_bottom + table.style.padding_top;
        let available_height = self.cursor.1 - frame_bottom - vertical_padding;
        let row_cursor = self.cursor;
        let mut row_height = 0.0;
        let frame_width = self.template.get_frame().width;
        // Calculate row's total columns based on cell spans (default span is 1).
        let columns = row
            .cells
            .iter()
            .map(|c: &Cell| c.span)
            .fold(0.0, |sum, span| sum + span);
        let span_width = frame_width / columns as f32;
        // First pass: check height of whole row to see if it fits the current page
        // before rendering it to current page. Otherwise open a new page.
        // Cell rects for drawing borders and backgrounds.
        let mut cell_rects: Vec<Rect> = Vec::new();
        // Collect content widths for horizontal alignment.
        let mut cell_content_widths: Vec<f32> = Vec::new();
        // Collect content heights for vertical alignment.
        let mut cell_content_heights: Vec<f32> = Vec::new();
        // Set the first cell's location to the beginning of row
        let mut rect_cursor = row_cursor;
        for cell in &row.cells {
            let cell_width = cell.span * span_width;
            let mut cell_content_width = 0.0;
            let mut cell_height = 0.0;
            for content in &cell.contents {
                let (actual_width, actual_height) =
                    content.wrap((cell_width - horizontal_padding, available_height));
                if actual_height > available_height {
                    if new_page {
                        return Err(JsValue::from_str(
                            "Cell content is too large to fit on page.",
                        ));
                    }
                    // Cell content doesn't fit, open new page
                    self.save_page();
                    // New row on page is always first row.
                    return self.draw_table_row(table, row, table_cursor, true, true);
                }
                // Add content height to cell height.
                cell_height += actual_height;
                // Content width
                if cell_content_width < actual_width {
                    cell_content_width = actual_width;
                }
            }
            // Adjust row height to max cell height and add vertical padding to height.
            if cell_height > row_height {
                row_height = cell_height;
            }
            let rect: Rect = Rect::new(rect_cursor.0, rect_cursor.1, cell_width, 0.0);
            cell_rects.push(rect);
            rect_cursor.0 += cell_width;
            // Add cell content width
            cell_content_widths.push(cell_content_width);
            // Add cell content height
            cell_content_heights.push(cell_height);
        }
        // Adjust all cell rects to row height
        for rect in &mut cell_rects {
            rect.h = row_height + vertical_padding;
        }

        let mut grid_lines: Vec<Line> = Vec::new();
        // Add top and bottom lines
        if let Some(r1) = cell_rects.first() {
            if let Some(r2) = cell_rects.last() {
                if is_first_row {
                    let top_line = Line::new(r1.x, r1.y, r2.x + r2.w, r2.y);
                    grid_lines.push(top_line);
                }
                let bottom_line = Line::new(r1.x, r1.y - r1.h, r2.x + r2.w, r2.y - r2.h);
                grid_lines.push(bottom_line);
            }
        }
        // Add vertical lines
        for (index, r) in cell_rects.iter().enumerate() {
            if index == 0 {
                let left_line = Line::new(r.x, r.y, r.x, r.y - r.h);
                grid_lines.push(left_line);
            }
            let right_line = Line::new(r.x + r.w, r.y, r.x + r.w, r.y - r.h);
            grid_lines.push(right_line);
        }
        // Set the first cell's location to the beginning of row
        let mut cell_cursor = (row_cursor.0, row_cursor.1);
        for (index, cell) in row.cells.iter().enumerate() {
            let cell_width = cell.span * span_width;
            // Background color fill
            if let Some(bg_color) = cell.style.background_color {
                let rect = cell_rects[index];
                self.fill_rect(rect, bg_color);
            }
            // Check for vertical alignment
            let cell_content_height = cell_content_heights[index];
            let offset_top = match table.style.vertical_align {
                VerticalAlign::Middle => {
                    (row_height + vertical_padding - cell_content_height) / 2.0
                }
                VerticalAlign::Bottom => {
                    row_height + vertical_padding - cell_content_height - table.style.padding_bottom
                }
                _ => table.style.padding_top,
            };
            // Set vertical offset
            self.cursor.1 = cell_cursor.1 - offset_top;
            self.cursor.0 = cell_cursor.0 + table.style.padding_left;
            for content in &cell.contents {
                content.draw(self, cell_width - horizontal_padding)?
            }
            cell_cursor.0 += cell_width;
            self.set_cursor(cell_cursor.0, cell_cursor.1);
        }
        self.set_cursor(row_cursor.0, row_cursor.1 - row_height - vertical_padding);
        // Draw grid lines if so configured
        if table.style.grid_visible {
            self.draw_lines(grid_lines, table);
        }
        Ok(())
    }
    pub fn draw_table(&mut self, table: &Table) -> Result<(), JsValue> {
        let table_cursor = self.cursor;
        // Render rows individually (may render on separate pages).
        let mut is_first_row = true;
        for row in &table.rows {
            self.draw_table_row(table, row, table_cursor, is_first_row, false)?;
            is_first_row = false;
        }
        self.set_cursor(table_cursor.0, self.cursor.1);
        Ok(())
    }
    pub fn draw_image(
        &mut self,
        image: &Image,
        new_page: bool,
        available_width: f32,
    ) -> Result<(), JsValue> {
        // add image to canvas images first, then add transform to output
        // check first if image fits to this page..
        let frame_bottom = self.template.get_frame().y - self.template.get_frame().height;
        let width = if image.fit_width {
            available_width
        } else {
            image.width
        };
        let height = if image.fit_width {
            available_width / image.width * image.height
        } else {
            image.height
        };
        let pos_x = if !image.fit_width {
            match image.style.horizontal_align {
                HorizontalAlign::Left => self.cursor.0,
                HorizontalAlign::Center => self.cursor.0 + (available_width - image.width) / 2.0,
                _ => self.cursor.0 + available_width - image.width,
            }
        } else {
            self.cursor.0
        };
        if self.cursor.1 - image.height < frame_bottom {
            if new_page {
                return Err(JsValue::from_str("Image is too large to fit on page."));
            }
            self.save_page();
            return self.draw_image(image, true, available_width);
        }

        let image_id = self.doc.get_image_id();
        let pdf_image = PDFImage::new(image_id, image.width, image.height, &image.data);
        let image_name = pdf_image.get_uid();
        self.images.push(pdf_image);
        self.set_cursor(self.cursor.0, self.cursor.1 - height);
        self.save_state();
        self.translate(pos_x, self.cursor.1);
        let mut stream = Vec::new();
        writeln!(stream, "{} 0 0 {} 0 0 cm", width, height).unwrap();
        writeln!(stream, "/{} Do", image_name).unwrap();
        self.output.write_all(&stream).unwrap();
        self.restore_state();
        Ok(())
    }
    pub fn draw_text(
        &mut self,
        paragraph: &Paragraph,
        wrapped: &[Vec<TextSpan>],
        available_width: f32,
    ) -> Result<(), JsValue> {
        self.doc.add_font(paragraph.font); // font gets added only if it doesn't exist yet
        let leading = paragraph.style.leading;
        let padding_top = paragraph.style.padding.0;
        let padding_left = paragraph.style.padding.1;
        let padding_bottom = paragraph.style.padding.2;
        self.cursor = (self.cursor.0, self.cursor.1 - leading - padding_top);
        self.save_state();
        self.translate(self.cursor.0 + padding_left, self.cursor.1);
        self.save_state();
        let color = paragraph.style.color;
        self.set_fill_color(color.r, color.g, color.b);
        let mut out_text: Vec<u8> = Vec::new();
        if let Some(bullet) = &paragraph.style.bullet {
            let mut bullet_text: Vec<u8> = Vec::new();
            write!(bullet_text, "-{} 0 Td ", paragraph.style.bullet_indent).unwrap();
            bullet_text.extend(TextSpan::encode_text(&bullet));
            let mut stream = Vec::new();
            write!(
                stream,
                "BT 1 0 0 1 0 2 Tm /{} {} Tf {} TL ",
                paragraph.font.get_ref(),
                paragraph.font_size,
                leading
            )
            .unwrap();
            stream.write_all(&bullet_text).unwrap();
            writeln!(stream, " ET").unwrap();
            self.output.write_all(&stream).unwrap();
        }

        let mut next_page_lines: Vec<Vec<TextSpan>> = Vec::new();
        let frame_bottom = self.template.get_frame().y - self.template.get_frame().height;
        let mut break_page = false;
        for line in wrapped {
            let mut line_width: f32 = 0.0;
            let mut width_offset: f32 = 0.0;
            // check first if we have to write to next page
            if self.cursor.1 < frame_bottom {
                break_page = true;
                next_page_lines.push(line.clone());
            } else {
                match paragraph.style.align {
                    HorizontalAlign::Center => {
                        for span in line {
                            line_width += span.get_width(paragraph.font, paragraph.font_size);
                        }
                        width_offset = (available_width - line_width) / 2.0;
                        write!(out_text, " {} 0 Td ", width_offset).unwrap();
                    }
                    HorizontalAlign::Right => {
                        for span in line {
                            line_width += span.get_width(paragraph.font, paragraph.font_size);
                        }
                        width_offset = available_width - line_width;
                        write!(out_text, " {} 0 Td ", width_offset).unwrap();
                    }
                    _ => (),
                }
                let mut _x: f32 = self.cursor.0;
                let mut _y: f32 = self.cursor.1;

                let mut text_color_changed = false;
                for span in line {
                    let span_width = span.get_width(paragraph.font, paragraph.font_size);
                    match &span.tag {
                        Tag::Link { url } => {
                            let annot =
                                LinkAnnotation::new(&url, _x, _y, _x + span_width, _y + leading);
                            self.link_annotations.push(annot);
                            out_text.extend(format!(" {} {} {} rg ", 0.2, 0.2, 1.0).as_bytes());
                            text_color_changed = true;
                        }
                        _ => {
                            // Only change text color when necessary.
                            if text_color_changed {
                                out_text.extend(
                                    format!(" {} {} {} rg ", color.r, color.g, color.b).as_bytes(),
                                );
                                text_color_changed = false;
                            }
                        }
                    }
                    out_text.extend(span.encoded_text());
                    _x += span_width;
                }
                write!(out_text, " T* ").unwrap();
                // Reset any offsets after printing a line
                if width_offset != 0.0 {
                    write!(out_text, " {} 0 Td ", -width_offset).unwrap();
                }
                self.cursor = (self.cursor.0, self.cursor.1 - leading);
            }
        }
        // move up one leading to count for one row of text
        self.cursor = (self.cursor.0, self.cursor.1 + leading - padding_bottom);
        let mut stream = Vec::new();
        write!(
            stream,
            "BT 1 0 0 1 0 2 Tm /{} {} Tf {} TL ",
            paragraph.font.get_ref(),
            paragraph.font_size,
            leading
        )
        .unwrap();
        stream.write_all(&out_text).unwrap();
        writeln!(stream, " ET").unwrap();
        self.output.write_all(&stream).unwrap();
        self.restore_state();
        self.restore_state();
        if break_page {
            self.save_page();
            return self.draw_text(paragraph, &next_page_lines, available_width);
        }
        Ok(())
    }
    pub fn build(&mut self) -> Result<Vec<u8>, JsValue> {
        self.save_page();
        self.doc.save_document(&self.template)
    }
}

#[cfg(test)]
mod tests {
    use super::super::units::A4;
    use super::*;

    #[test]
    fn test_initial_canvas() {
        let tpl = PageTemplate::new(A4, 50.0, 50.0, 50.0, 50.0);
        let canvas = Canvas::new(&tpl);
        let output = "1 0 0 1 0 0 cm  BT /F1 12 Tf 14.4 TL ET\n".as_bytes();
        assert_eq!(canvas._get_output(), output);
    }
}
