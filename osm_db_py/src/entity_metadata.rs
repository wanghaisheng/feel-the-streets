use osm_db::entity_metadata::{EntityMetadata, Field, Enum};
use pyo3::exceptions;
use pyo3::prelude::*;
use std::collections::HashMap;

#[pyclass(name=EntityMetadata)]
pub struct PyEntityMetadata {
    inner: EntityMetadata,
}
#[pymethods]
impl PyEntityMetadata {
    #[staticmethod]
    pub fn for_discriminator(discriminator: &str) -> PyResult<Self> {
        match EntityMetadata::for_discriminator(discriminator) {
            Some(metadata) => Ok(Self { inner: metadata }),
            None => Err(exceptions::KeyError::py_err(format!(
                "No entity with discriminator {}.",
                discriminator
            ))),
        }
    }
    #[getter]
    pub fn discriminator(&self) -> &str {
        self.inner.discriminator.as_str()
    }

    #[getter]
    pub fn fields(&self) -> HashMap<&String, PyField> {
        self.inner
            .fields
            .iter()
            .map(|(k, v)| (k, PyField { inner: v.clone() }))
            .collect()
    }

    #[getter]
    pub fn display_template(&self) -> Option<&String> {
        self.inner.display_template.as_ref()
    }

    #[getter]
    pub fn all_fields(&self) -> HashMap<String, PyField> {
        self.inner.all_fields().into_iter().map(|(k, f)| (k, PyField{inner: f.clone()})).collect()
    }

    #[getter]
    pub fn parent_metadata(&self) -> Option<Self> {
        self.inner.parent_metadata().map(|m| Self{inner: m})
    }
}

#[pyclass(name=Field)]
pub struct PyField {
    inner: Field,
}

#[pymethods]
impl PyField {
    #[getter]
    pub fn type_name(&self) -> &str {
        self.inner.type_name.as_str()
    }

    #[getter]
    pub fn required(&self) -> bool {
        self.inner.required
    }
}

#[pyclass(name=Enum)]
pub struct PyEnum {
    inner: Enum
}

#[pymethods]
impl PyEnum {
    
    #[staticmethod]
    pub fn all_known() -> Vec<&'static String> {
        Enum::all_known()
    }

    #[staticmethod]
    pub fn with_name(name: &str) -> Option<Self> {
        Enum::with_name(name).map(|e| PyEnum{inner: e})
    }

    #[getter]
    pub fn name(&self) -> &str {
        self.inner.name.as_str()
    }

    pub fn value_for_name(&self, name: &str) -> Option<i32> {
        self.inner.value_for_name(name).map(|v| *v)
    }
    pub fn name_for_value(&self, value: i32) -> Option<&'static String> {
self.inner.name_for_value(value)
    }
}