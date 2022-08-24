#[allow(dead_code)]
pub mod c_elements;
#[allow(dead_code)]
pub mod wasm_elements;

pub mod components;

use std::str::FromStr;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

pub fn get_number_version() -> (usize, usize, usize) {
    let version_splitted: Vec<&str> = VERSION.split(".").collect();
    (
        usize::from_str(version_splitted[0]).unwrap(),
        usize::from_str(version_splitted[1]).unwrap(),
        usize::from_str(version_splitted[2]).unwrap(),
    )
}
