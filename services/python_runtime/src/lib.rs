mod ipc;

use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

use ipc::AiCoreClient;

fn py_err(error: impl std::fmt::Display) -> PyErr {
    PyRuntimeError::new_err(error.to_string())
}

#[pyfunction]
fn send_message(service: String, payload: Vec<u8>) -> PyResult<Vec<u8>> {
    AiCoreClient::from_env()
        .send_message(service, payload)
        .map_err(py_err)
}

#[pyfunction]
#[pyo3(signature = (goal, session_id=None, user_id=None))]
fn create_plan(goal: String, session_id: Option<String>, user_id: Option<String>) -> PyResult<Vec<u8>> {
    AiCoreClient::from_env()
        .encode_plan_request(goal, session_id, user_id)
        .map_err(py_err)
}

#[pyfunction]
fn call_tool(tool_name: String, parameters_json: String) -> PyResult<Vec<u8>> {
    let parameters: serde_json::Value = serde_json::from_str(&parameters_json).map_err(py_err)?;
    AiCoreClient::from_env()
        .encode_tool_call(tool_name, parameters)
        .map_err(py_err)
}

#[pymodule]
fn polymera_native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(send_message, m)?)?;
    m.add_function(wrap_pyfunction!(create_plan, m)?)?;
    m.add_function(wrap_pyfunction!(call_tool, m)?)?;
    Ok(())
}
