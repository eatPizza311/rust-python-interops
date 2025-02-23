use pyo3::prelude::*;
use pyo3::types::{PyInt, PyList};

#[pyfunction]
// TODO: Implement a function that returns a list containing the first `n` numbers in Fibonacci's sequence.
fn fibonacci(n: u64) -> Vec<u64> {
    let mut result = Vec::from([0, 1]);
    if n < 2 {
        return result[..n as usize].to_vec();
    } else {
        for i in 2..n {
            let len = result.len();
            let next = result[len - 1] + result[len - 2];
            result.push(next);
        }
    }
    result
}

#[pymodule]
fn output(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(fibonacci, m)?)?;
    Ok(())
}
