// rustimport:pyo3

use pyo3::prelude::*;

#[pyfunction]
pub fn say_hello() {
    println!("Hello from ruspy, implemented in Rust!")
}
