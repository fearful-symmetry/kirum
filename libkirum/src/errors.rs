use rhai::EvalAltResult;



#[derive(thiserror::Error, Debug)]
pub enum LangError {
    #[error("{0}")]
    ValidationError(String),
    #[error("error reading provided JSON input")]
    JSONImportError(#[source] std::io::Error),

    #[error("error parsing JSON input")]
    JSONSerdeError(#[source] serde_json::Error)
}

#[derive(thiserror::Error, Debug)]
#[error("error parsing phonetic value: {msg}; found {found}")]
pub struct PhoneticParsingError {
    pub msg: &'static str,
    pub found: String
}

#[derive(thiserror::Error, Debug)]
#[error("invalid part of speech value {found}")]
pub struct POSFromError {
    pub found: String
}

#[derive(thiserror::Error, Debug)]
#[error("could not parse dynamic type {dyn_type} into Lemma. Return must be an array of strings or string")]
pub struct LemmaFromError {
    pub dyn_type: String,
}

#[derive(thiserror::Error, Debug)]
pub enum TransformError {
    #[error("error evaluating Rhai script")]
    EvalError(#[from] Box<EvalAltResult>),
    #[error("could not parse return value from script")]
    ScriptReturnValueError(#[from] LemmaFromError)
}