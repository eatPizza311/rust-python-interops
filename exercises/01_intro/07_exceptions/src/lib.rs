use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::PyInt;

#[pyfunction]
// TODO: Implement a function that returns a list containing the first `n` numbers in Fibonacci's sequence.
//  It must raise a `TypeError` if `n` is not an integer or if it is less than 0.
fn fibonacci(n: Bound<'_, PyInt>) -> PyResult<Vec<u64>> {
    let n = n
        .extract::<usize>()
        .map_err(|_| PyTypeError::new_err("The value must be a positive interger"))?;
    let mut result: Vec<u64> = Vec::from([0, 1]);
    if n < 2 {
        Ok(result[..n].to_vec())
    } else {
        for _ in 2..n {
            let len = result.len();
            let next = result[len - 1] + result[len - 2];
            result.push(next);
        }
        Ok(result)
    }
}

#[pymodule]
fn exceptions(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(fibonacci, m)?)?;
    Ok(())
}
