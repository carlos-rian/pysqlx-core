use pyo3::{
    types::{PyAnyMethods, PyModule},
    Python,
};

pub fn tokio_runtime() -> &'static tokio::runtime::Runtime {
    use std::sync::OnceLock;
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| tokio::runtime::Runtime::new().unwrap())
}

pub fn _check_event_trio_event_loop(py: Python) -> bool {
    PyModule::import_bound(py, "trio")
        .ok()
        .and_then(|m| m.getattr("lowlevel").ok())
        .and_then(|lowlevel| lowlevel.call_method0("current_trio_token").ok())
        .is_some()
}
