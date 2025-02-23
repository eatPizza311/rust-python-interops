use pyo3::prelude::*;

#[pyfunction]
// TODO: Define a function that takes as input a vector of unsigned integers
//  and prints each number in the list.
fn print_number_list(numbers: Vec<u64>) {
    for num in numbers{
        println!("{num}");
    }
}

#[pymodule]
fn arguments(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(print_number_list, m)?)?;
    Ok(())
}
