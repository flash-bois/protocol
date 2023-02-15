mod utils;

//use core_lib::{add_points, sum, Point};
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
}

#[wasm_bindgen]
pub fn greet() {
    alert("Hello, wasm!");
}

// #[wasm_bindgen]
// pub fn add_no_points(a: i32, b: i32) -> i32 {
//     let p = Point { x: a, y: b };
//     let d = Point { x: a, y: b };
//     let result = add_points(p, d);
//     sum(result)
// }
