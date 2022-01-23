#[forbid(unsafe_code)]

mod field;
mod judges;

use wasm_bindgen::prelude::*;

pub struct Args {
    density: f32,
    solvable: bool,
    judge: judges::Judge,
    cheat: bool,
    autosave: bool,
    reset: bool,
}

#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, test-wasm!");
}
