extern crate console_error_panic_hook;
extern crate wasm_bindgen;
extern crate web_sys;

use rust_xlsxwriter::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello, {}!", name));
}

#[wasm_bindgen]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
pub fn xls_write() {
    let mut workbook = Workbook::new();
    // Create some formats to use in the worksheet.
    let bold_format = Format::new().set_bold();
    let decimal_format = Format::new().set_num_format("0.000");
    let date_format = Format::new().set_num_format("yyyy-mm-dd");
    let merge_format = Format::new()
        .set_border(FormatBorder::Thin)
        .set_align(FormatAlign::Center);

    // Add a worksheet to the workbook.
    let worksheet = workbook.add_worksheet();

    // Set the column width for clarity.
    worksheet.set_column_width(0, 22).unwrap();

    // Write a string without formatting.
    worksheet.write(0, 0, "Hello").unwrap();

    // Write a string with the bold format defined above.
    worksheet
        .write_with_format(1, 0, "World", &bold_format)
        .unwrap();

    // Write some numbers.
    worksheet.write(2, 0, 1).unwrap();
    worksheet.write(3, 0, 2.34).unwrap();

    // Write a number with formatting.
    worksheet
        .write_with_format(4, 0, 3.00, &decimal_format)
        .unwrap();

    // Write a formula.
    worksheet.write(5, 0, Formula::new("=SIN(PI()/4)")).unwrap();

    // Write a date.
    let date = ExcelDateTime::from_ymd(2023, 1, 25).unwrap();
    worksheet
        .write_with_format(6, 0, &date, &date_format)
        .unwrap();

    // Write some links.
    worksheet
        .write(7, 0, Url::new("https://www.rust-lang.org"))
        .unwrap();
    worksheet
        .write(8, 0, Url::new("https://www.rust-lang.org").set_text("Rust"))
        .unwrap();

    // Write some merged cells.
    worksheet
        .merge_range(9, 0, 9, 1, "Merged cells", &merge_format)
        .unwrap();

    // Save the file to disk.
    //workbook.save("D:\\demo.xlsx").unwrap();
}
