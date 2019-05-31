#![allow(dead_code)]

use std::collections::HashSet;
use std::io::Write;
use std::str;
use wasm_bindgen::prelude::*;

use super::encoders;
use super::font::Font;
use super::template::PageTemplate;

/// PDFDocument is the main type used when creating an actual document.
///
pub struct PDFDocument {
    pages: Vec<PDFPage>,
    page_counter: u16,
    image_counter: u16,
    fonts: HashSet<&'static Font>,
}

impl PDFDocument {
    pub fn new() -> PDFDocument {
        PDFDocument {
            pages: Vec::new(),
            page_counter: 1,
            image_counter: 0,
            fonts: HashSet::new(),
        }
    }
    pub fn add_font(&mut self, font: &'static Font) {
        self.fonts.insert(font);
    }
    pub fn add_page(&mut self, page: PDFPage) {
        self.pages.push(page);
        self.page_counter += 1;
    }
    pub fn get_image_id(&mut self) -> u16 {
        self.image_counter += 1;
        self.image_counter
    }
    pub fn save_document(&mut self, tpl: PageTemplate) -> Result<Vec<u8>, JsValue> {
        let mut pdf = PDFFile::new();
        let font_id = pdf.get_new_object_id();
        let mut font_resources = String::new();
        // delay adding font resources to document after root font object (1 0 R)
        let mut font_resource_objects: Vec<PDFObject> = Vec::new();
        for font in &self.fonts {
            let font_resource_id = pdf.get_new_object_id();
            font_resources += &format!("/{} {} 0 R ", font.get_ref(), font_resource_id);
            let font_resource_obj = PDFObject::new(
            &format!("/BaseFont /{} /Encoding /WinAnsiEncoding /Name /{} /Subtype /Type1 /Type /Font", 
                font.get_name(), font.get_ref()),
                font_resource_id
            );
            font_resource_objects.push(font_resource_obj);
        }
        // Root font object
        let font_obj = PDFObject::new(&font_resources, font_id);
        pdf.add_object(&font_obj);
        // Font resource objects
        for font_resource_obj in font_resource_objects {
            pdf.add_object(&font_resource_obj);
        }
        let root_id = pdf.get_new_object_id();
        let pages_id = pdf.get_new_object_id();
        let root_obj = PDFObject::new(&format!("/Type /Catalog /Pages {} 0 R", pages_id), root_id);
        pdf.add_object(&root_obj);
        let mut kids = Vec::new();
        kids.write_all(b"[ ").unwrap();
        // get page ids
        for page in &mut self.pages {
            // page images
            for image in &mut page.images {
                image.object_id = pdf.get_new_object_id();
            }
            let page_id = pdf.get_new_object_id();
            page.page_id = page_id;
            write!(kids, "{} 0 R ", page_id).unwrap();
        }
        kids.write_all(b"]").unwrap();
        // write pages array
        let pages_obj = PDFObject::new(
            &format!(
                "/Type /Pages /Kids {} /Count {}",
                str::from_utf8(&kids).unwrap(),
                self.pages.len()
            ),
            pages_id,
        );
        pdf.add_object(&pages_obj);
        // Retrieve ids for page contents
        for page in &mut self.pages {
            let content_id = pdf.get_new_object_id();
            page.content_id = content_id;
        }
        for page in &self.pages {
            let mut x_objects: Vec<String> = Vec::new();
            for image in &page.images {
                // print all image resources here and get identifiers for next phase
                let bytes = encoders::ascii85::encode(&image.contents).unwrap();
                let mut output = Vec::new();
                writeln!(output, "{} 0 obj", image.object_id).unwrap();
                writeln!(
                    output,
                    "<<  
/Type /XObject
/Subtype /Image 
/Height {} 
/Width {} 
/ColorSpace /DeviceRGB
/BitsPerComponent 8
/Filter [/ASCII85Decode /DCTDecode]
/Length {} 
>>",
                    image.height,
                    image.width,
                    bytes.len() + 2 // ~> + 2
                )
                .unwrap();
                writeln!(output, "stream").unwrap();
                output.write_all(&bytes).unwrap();
                write!(output, "~>").unwrap();
                writeln!(output, "endstream").unwrap();
                writeln!(output, "endobj").unwrap();
                pdf.add(&output);
                x_objects.push(format!("/{} {} 0 R", image.get_uid(), image.object_id));
            }
            let x_object = if x_objects.is_empty() {
                String::new()
            } else {
                format!("/XObject <<\n{}\n>>", x_objects.join(" "))
            };
            let content_id = page.content_id;
            let page_obj = PDFObject::new(
                &format!(
                    "/Type /Page 
/Parent {} 0 R 
/Resources <<
    /Font {} 0 R 
    /ProcSet [ /PDF /Text {}] 
    {}
>> 
/MediaBox [0 0 {} {}] 
/Contents {} 0 R ",
                    pages_id,
                    font_id,
                    if x_objects.is_empty() { "" } else { "/ImageC" },
                    x_object,
                    tpl.get_size().0,
                    tpl.get_size().1,
                    content_id
                ),
                page.page_id,
            );
            pdf.add_object(&page_obj);
        }
        // add page contents here
        for page in &self.pages {
            let content_id = page.content_id;
            let stream = &page.contents;
            let stream = encoders::zlib::encode(&stream).unwrap();
            let stream = encoders::ascii85::encode(&stream).unwrap();
            let mut output = Vec::new();
            writeln!(
                output,
                "{} 0 obj\n<<\n/Filter [/ASCII85Decode /FlateDecode] /Length {}\n>>",
                content_id,
                stream.len() + 2 // ~> + 2
            )
            .unwrap();
            writeln!(output, "stream").unwrap();
            output.write_all(&stream).unwrap();
            write!(output, "~>").unwrap(); // ascii85 stream end marker
            writeln!(output, "endstream").unwrap();
            writeln!(output, "endobj").unwrap();
            pdf.add(&output);
        }
        pdf.add_trailer(root_id);
        Ok(pdf.output())
    }
}

pub struct PDFPage {
    contents: Vec<u8>,
    page_id: u16,
    content_id: u16,
    pub images: Vec<PDFImage>,
}

impl PDFPage {
    pub fn new() -> PDFPage {
        PDFPage {
            contents: Vec::new(),
            page_id: 0,
            content_id: 0,
            images: Vec::new(),
        }
    }
    pub fn set_contents(&mut self, input: &[u8]) {
        self.contents = input.to_vec();
    }
    pub fn set_images(&mut self, input: &[PDFImage]) {
        self.images = input.to_vec();
    }
}

pub struct PDFObject {
    contents: Vec<u8>,
    // id: u16,
}

impl PDFObject {
    pub fn new(text: &str, id: u16) -> Self {
        let mut output = Vec::new();
        writeln!(output, "{} 0 obj\n<<\n{}\n>>\nendobj", id, text).unwrap();
        PDFObject {
            contents: output,
            // id,
        }
    }
}

/// PDFFile is created by PDFDocument.
/// Adds bytes to contents, keeps track of offsets and object count.
pub struct PDFFile {
    contents: Vec<u8>,
    object_counter: u16,
    offsets: Vec<u32>,
}

impl PDFFile {
    pub fn new() -> Self {
        let mut output = Vec::new();
        output.write_all(b"%PDF-1.4").unwrap();
        output
            .write_all(b"\n%\x93\x8C\x8B\x9E PackPDF WASM library\n")
            .unwrap();
        PDFFile {
            contents: output,
            object_counter: 1,
            offsets: Vec::new(),
        }
    }
    /// Append bytes to contents and current offset
    pub fn add(&mut self, bytes: &[u8]) {
        self.offsets.push(self.contents.len() as u32);
        self.contents.write_all(bytes).unwrap();
    }
    /// Give new object id (this id should be consumed)
    pub fn get_new_object_id(&mut self) -> u16 {
        let id = self.object_counter;
        self.object_counter += 1;
        id
    }
    pub fn add_object(&mut self, obj: &PDFObject) {
        self.offsets.push(self.contents.len() as u32);
        self.contents.write_all(&obj.contents[..]).unwrap();
    }
    fn add_cross_reference_table(&mut self) {
        for offset in &self.offsets {
            writeln!(self.contents, "{:010} 00000 n ", offset).unwrap();
        }
    }
    fn add_trailer(&mut self, root_id: u16) {
        //let num_objects = self.offsets.len() + 1;
        let num_objects = self.object_counter;
        let xref_start_offset = self.contents.len();
        writeln!(self.contents, "xref").unwrap();
        writeln!(self.contents, "0 {}", num_objects).unwrap();
        writeln!(self.contents, "0000000000 65535 f").unwrap();
        self.add_cross_reference_table();
        writeln!(
            self.contents,
            "trailer <</Size {} /Root {} 0 R>>",
            num_objects, root_id
        )
        .unwrap();
        writeln!(self.contents, "startxref").unwrap();
        writeln!(self.contents, "{}", xref_start_offset).unwrap();
        writeln!(self.contents, "%%EOF").unwrap();
    }
    pub fn output(&self) -> Vec<u8> {
        self.contents.clone()
    }
}

#[derive(Clone)]
pub struct PDFImage {
    width: f32,
    height: f32,
    contents: Vec<u8>,
    image_id: u16,  // image identifier
    object_id: u16, // pdf object id
}

impl PDFImage {
    pub fn new(image_id: u16, width: f32, height: f32, bytes: &[u8]) -> PDFImage {
        PDFImage {
            width,
            height,
            contents: bytes.to_vec(),
            image_id,
            object_id: 0,
        }
    }
    pub fn get_uid(&self) -> String {
        format!("Im{}", self.image_id)
    }
}
