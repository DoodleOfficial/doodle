/**
 * @file lib.rs
 * @author Krisna Pranav
 * @brief lib
 * @version 1.0
 * @date 2024-11-25
 *
 * @copyright Copyright (c) 2024 Doodle Developers, Krisna Pranav
 *
 */

use pyo3::prelude::*;

#[pymodule]
mod leechy {
    use super::*;
    use ::leechy as lchy;

    #[pyclass]
    struct Engine {
        inner: lchy::Engine,
    }

    #[pymethods]
    impl Engine {
        #[new]
        #[pyo3(signature = (name = "google"))]
        fn new(name: &str) -> PyResult<Self> {
            match lchy::Engine
        }
    }
}