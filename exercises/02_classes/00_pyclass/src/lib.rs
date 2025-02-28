use pyo3::prelude::*;

// TODO: Create a new Python class named `ShoppingOrder` with the following attributes:
//  - `price` (positive integer)
//  - `quantity` (positive integer)
//  - `name` (string)
//  Expose a `new_order` function to create a new `ShoppingOrder` instance.
//  It shouldn't be possible to modify the name or the price after the object is created.

#[pyclass]
struct ShoppingOrder {
    #[pyo3(get)]
    price: u32,
    #[pyo3(get, set)]
    quantity: u32,
    #[pyo3(get)]
    name: String,
}

#[pyfunction]
fn new_order(name: String, price: u32, quantity: u32) -> ShoppingOrder {
    ShoppingOrder {
        name,
        price,
        quantity,
    }
}

#[pymodule]
fn pyclass(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(new_order, m)?)?;
    m.add_class::<ShoppingOrder>()?;
    Ok(())
}
