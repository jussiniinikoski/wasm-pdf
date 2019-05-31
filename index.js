const pack_pdf = import('./pkg/pack_pdf');

const createPDF = (jsDocument) => {
    pack_pdf.then(pdf => {
        const imagePaths = parseJsDoc(jsDocument.contents);
        fetchImagePaths(imagePaths).then((imgData) => {
            // add base64 encoded bytes to document
            jsDocument.image_data = {};
            // add image widths and heights
            jsDocument.image_widths = {};
            jsDocument.image_heights = {};
            imgData.map(d => {
                jsDocument.image_data[d.path] = d.data;
                jsDocument.image_widths[d.path] = d.width;
                jsDocument.image_heights[d.path] = d.height;
            });
            pdf.run(jsDocument);
        });
        //pdf.print_document(jsDocument);
    }).catch(console.error)
};
window.createPDF = createPDF;

// convert list of image paths
const fetchImagePaths = paths => Promise.all(paths.map(p => imageBytes(p)));

// convert single image path
const imageBytes = (url) => {
    return new Promise(resolve => {
        let img = new Image();
        img.onload = () => {
            let canvas = document.createElement("canvas");
            canvas.width = img.width;
            canvas.height = img.height;
            let ctx = canvas.getContext("2d");
            ctx.drawImage(img, 0, 0);
            let dataURL = canvas.toDataURL("image/jpeg", 0.8);
            resolve({
                path: url,
                data: stripDataURI(dataURL),
                width: img.width,
                height: img.height
            });
        };
        img.onerror = () => resolve({
            path,
            status: 'error'
        });
        img.src = url;
    });
};

const BASE64_MARKER = ';base64,';

const stripDataURI = (dataURI) => {
    const base64Index = dataURI.indexOf(BASE64_MARKER) + BASE64_MARKER.length;
    return dataURI.substring(base64Index);
};

const parseJsDoc = (obj) => {
    let results = [];
    for (let k in obj) {
        if (obj[k] && typeof obj[k] === 'object') {
            results = results.concat(parseJsDoc(obj[k]));
        } else {
            if (k === "obj_type" && obj[k] === "Image") {
                if (obj["params"] && obj["params"]["src"]) {
                    results.push(obj.params.src);
                }
            }
        }
    }
    return results;
}