use std::fmt;

use common::MalData;

#[derive(Debug, Clone)]
pub enum EvalError {
    General(String)
}

impl From<&'static str> for EvalError {
    fn from(err: &str) -> Self {
        EvalError::General(err.to_string())
    }
}

impl From<String> for EvalError {
    fn from(err: String) -> Self {
        EvalError::General(err)
    }
}

impl<'e> fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            EvalError::General(ref err_msg) => {
                write!(f, "{}", err_msg)
            }
        }
    }
}

pub type MalEvalResult = Result<MalData, EvalError>;

