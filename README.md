# WASM-PDF
## Generates PDF files directly in the browser with JavaScript and WASM (WebAssembly).

[![Build Status](https://travis-ci.org/jussiniinikoski/wasm-pdf.svg?branch=master)](https://travis-ci.org/jussiniinikoski/wasm-pdf)

Idea here is to push all the work involved in creating a PDF to the browser side, instead of using precious server resources.

## Demo

Example that generates a sample PDF document from JSON file. Title in PDF is changed dynamically (in JavaScript) to show current date. **Note:** this example routes directly to the PDF data blob, so 2 backsteps are needed to return to this page.

**[View Example 1](https://jussiniinikoski.github.io/wasm-pdf-demo/)**

Another example that generates 400 paragraphs of Lorem Ipsum and shows a download link when document has been generated (which is instantly 🙂). Also shows page numbers.

**[View Example 2](https://jussiniinikoski.github.io/wasm-pdf-demo/example2.html)**

## Features

* PDF contents are described as a JavaScript object and that gets passed to WASM-module, which generates the output
* Customizable output handler (e.g. load blob URL to a link or directly to window)
* Currently supported elements include:
  * Paragraphs/text elements with basic fonts (Helvetica, Times, Courier)
  * Images loaded from URL (converted automatically to bytes)
  * Tables with rows and cells
  * Spacers (they just eat space)
  * Paths with points, strokes, fills and alignment
  * Basic styling (all styling parameters are optional)
  * Custom template size (defaults to A4 portrait with 50 px margins)
  * Page numbers and stationary text can be included in template

## Example input (JSON)

```json
{
    "title": "Example Document",
    "contents": [{
            "obj_type": "Paragraph",
            "params": {
                "text": "Hello World!",
                "font_size": 18,
                "leading": 24,
                "align": "center",
                "font_name": "Helvetica-Bold"
            }
        }
    ]
}
```

## How to generate an example PDF

### Option 1 (with Rust)
* First install [the Rust compiler](https://www.rust-lang.org)
* Clone this repo:```git clone https://github.com/jussiniinikoski/wasm-pdf.git```
* Change to directory: ```cd wasm-pdf```
* Install JavaScript libraries: ```npm install```
* Launch the local development server: ```npm run serve```
* Open your browser and visit the url provided by the server, usually ```http://localhost:8080```

### Option 2 (with JavaScript only / npm package)
* Go to [starter template repo](https://github.com/jussiniinikoski/wasm-pdf-app/)

## Special Thanks

* [rustwasm/wasm-bindgen](https://github.com/rustwasm/wasm-bindgen)
* [kaj/rust-pdf](https://github.com/kaj/rust-pdf)
* [emreyaren/zero85](https://github.com/emreyaren/zero85)

## License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)
