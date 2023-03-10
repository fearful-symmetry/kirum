
#[derive(thiserror::Error, Debug)]
pub enum LangError {
    #[error("{0}")]
    ValidationError(String),
    #[error("error reading provided JSON input")]
    JSONImportError(#[source] std::io::Error),

    #[error("error parsing JSON input")]
    JSONSerdeError(#[source] serde_json::Error)
}