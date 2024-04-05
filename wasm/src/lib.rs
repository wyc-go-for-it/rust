extern crate console_error_panic_hook;
extern crate wasm_bindgen;
//extern crate web_sys;

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
pub struct WycWorkSheet{
    inner:Worksheet,
}

#[wasm_bindgen]
pub struct WycWorkBook{
    inner:Workbook,
}

#[wasm_bindgen]
impl WycWorkSheet{
    #[wasm_bindgen(constructor)]
    pub fn new()->Self{
        WycWorkSheet{
            inner:Worksheet::new(),
        }
    }

    pub fn writ_string(&mut self,row:u32,col:u16,str:&str){
        let _ = self.inner.write_string(row, col, str);
    }

    pub fn writ_string_bold(&mut self,row:u32,col:u16,str:&str){
        let _ = self.inner.write_string_with_format(row, col, str,&Format::new().set_bold());
    }

    pub fn writ_int(&mut self,row:u32,col:u16,str:f64){
        let _ = self.inner.write_number(row, col, str);
        
    }
}

#[wasm_bindgen]
impl WycWorkBook{
    #[wasm_bindgen(constructor)]
    pub fn new()->Self{
        WycWorkBook{
            inner:Workbook::new(),
        }
    }

    pub fn add_worksheet(&mut self,wks:WycWorkSheet){
        self.inner.push_worksheet(wks.inner);
    }

    pub fn save(&mut self)->Box<[u8]>{
        self.inner.save_to_buffer().unwrap().into_boxed_slice()
    }
}

   