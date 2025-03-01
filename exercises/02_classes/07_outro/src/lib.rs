// TODO: Define a base class named `Discount`, with a `percentage` attribute.
//  It should be possible to access the `percentage` attribute of a `Discount`.
//  It should also be possible to modify the `percentage` attribute of a `Discount`.
//  It must be enforced that the `percentage` attribute is a float between 0. and 1.
//  Then define two subclasses:
//  - `SeasonalDiscount` that inherits from `Discount` with two additional attributes, `to` and `from_`.
//    `from_` is a datetime object that represents the start of the discount period.
//    `to` is a datetime object that represents the end of the discount period.
//     Both `from_` and `to` should be accessible and modifiable.
//     The class should enforce that `from` is before `to`.
//  - `CappedDiscount` that inherits from `Discount` with an additional attribute `cap`.
//    `cap` is a float that represents the maximum discount (in absolute value) that can be applied.
//    It should be possible to access and modify the `cap` attribute.
//    The class should enforce that `cap` is a non-zero positive float.
//
// All classes should have a method named `apply` that takes a price (float) as input and
// returns the discounted price.
// `SeasonalDiscount` should raise an `ExpiredDiscount` exception if `apply` is called but
// the current date is outside the discount period.
use chrono::{DateTime, Utc};
use pyo3::{
    create_exception,
    exceptions::{PyException, PyValueError},
    prelude::*,
};

#[pyclass(subclass)]
struct Discount {
    #[pyo3(get)]
    percentage: f64,
}

fn check_percentage(perc: f64) -> PyResult<()> {
    if !(0.0..=1.0).contains(&perc) {
        return Err(PyValueError::new_err("Percentage must be between 0 and 1"));
    }
    Ok(())
}
#[pymethods]
impl Discount {
    #[new]
    fn new(percentage: f64) -> PyResult<Self> {
        check_percentage(percentage)?;
        Ok(Self { percentage })
    }

    #[setter]
    fn set_percentage(&mut self, percentage: f64) -> PyResult<()> {
        check_percentage(percentage)?;
        self.percentage = percentage;
        Ok(())
    }

    fn apply(&self, price: f64) -> f64 {
        (1. - self.percentage) * price
    }
}

#[pyclass(extends=Discount)]
struct SeasonalDiscount {
    #[pyo3(get)]
    to: DateTime<Utc>,
    #[pyo3(get)]
    from_: DateTime<Utc>,
}

fn check_duration(from: DateTime<Utc>, to: DateTime<Utc>) -> PyResult<()> {
    if from >= to {
        return Err(PyValueError::new_err(
            "`from_` date must be before `to` date",
        ));
    }
    Ok(())
}

create_exception!(outro2, ExpiredDiscount, PyException);

#[pymethods]
impl SeasonalDiscount {
    #[new]
    fn new(
        percentage: f64,
        from_: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> PyResult<PyClassInitializer<Self>> {
        check_duration(from_, to)?;
        let discount = Discount::new(percentage)?;
        let seasonal = SeasonalDiscount { from_, to };

        Ok(PyClassInitializer::from(discount).add_subclass(seasonal))
    }

    #[setter]
    fn set_from_(&mut self, from_: DateTime<Utc>) -> PyResult<()> {
        check_duration(from_, self.to)?;
        self.from_ = from_;
        Ok(())
    }

    #[setter]
    fn set_to(&mut self, to: DateTime<Utc>) -> PyResult<()> {
        check_duration(self.from_, to)?;
        self.to = to;
        Ok(())
    }

    fn apply(self_: PyRef<'_, Self>, price: f64) -> PyResult<f64> {
        let now = Utc::now();
        if now < self_.from_ || now > self_.to {
            return Err(ExpiredDiscount::new_err("The discount is no longer active"));
        }
        Ok(self_.as_super().apply(price))
    }
}

#[pyclass(extends=Discount)]
struct CappedDiscount {
    #[pyo3(get)]
    cap: f64,
}

#[pymethods]
impl CappedDiscount {
    #[new]
    fn new(percentage: f64, cap: f64) -> PyResult<PyClassInitializer<Self>> {
        if cap <= 0. {
            return Err(PyValueError::new_err("Cap must be a positive number"));
        }
        let discount = Discount::new(percentage)?;
        let capped = CappedDiscount { cap };
        Ok(PyClassInitializer::from(discount).add_subclass(capped))
    }

    #[setter]
    fn set_cap(&mut self, cap: f64) -> PyResult<()> {
        if cap <= 0. {
            return Err(PyValueError::new_err("Cap must be a positive number"));
        }
        self.cap = cap;
        Ok(())
    }

    fn apply(self_: PyRef<'_, Self>, price: f64) -> f64 {
        let discounted = self_.as_super().apply(price);
        if price - discounted > self_.cap {
            price - self_.cap
        } else {
            discounted
        }
    }
}

#[pymodule]
fn outro2(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<Discount>()?;
    m.add_class::<SeasonalDiscount>()?;
    m.add_class::<CappedDiscount>()?;
    m.add("ExpiredDiscount", m.py().get_type::<ExpiredDiscount>())?;
    Ok(())
}
