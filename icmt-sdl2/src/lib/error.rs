//! Errors generated from the mini terminal.

use log::error;

/// Result from mini terminal.
pub type IcmtResult<X> = Result<X, IcmtError>;

/// Errors from the mini terminal, or its subcomponents.
#[derive(Debug, Clone)]
pub enum IcmtError {
    Candid(std::sync::Arc<candid::Error>),
    Agent(), /* Clone => Agent(ic_agent::AgentError) */
    String(String),
    Engiffen(), /* Clone => engiffen::Error */
    RingKeyRejected(ring::error::KeyRejected),
    RingUnspecified(ring::error::Unspecified),
    FromHexError(hex::FromHexError),
}
impl std::convert::From<hex::FromHexError> for IcmtError {
    fn from(fhe: hex::FromHexError) -> Self {
        IcmtError::FromHexError(fhe)
    }
}

impl std::convert::From<ic_agent::AgentError> for IcmtError {
    fn from(ae: ic_agent::AgentError) -> Self {
        error!("Detected agent error: {:?}", ae);
        /*IcmtError::Agent(ae)*/
        IcmtError::Agent()
    }
}

impl std::convert::From<candid::Error> for IcmtError {
    fn from(e: candid::Error) -> Self {
        IcmtError::Candid(std::sync::Arc::new(e))
    }
}

impl<T> std::convert::From<std::sync::mpsc::SendError<T>> for IcmtError {
    fn from(_s: std::sync::mpsc::SendError<T>) -> Self {
        IcmtError::String("send error".to_string())
    }
}
impl std::convert::From<std::sync::mpsc::RecvError> for IcmtError {
    fn from(_s: std::sync::mpsc::RecvError) -> Self {
        IcmtError::String("recv error".to_string())
    }
}
impl std::convert::From<std::io::Error> for IcmtError {
    fn from(_s: std::io::Error) -> Self {
        IcmtError::String("IO error".to_string())
    }
}
impl std::convert::From<String> for IcmtError {
    fn from(s: String) -> Self {
        IcmtError::String(s)
    }
}
impl std::convert::From<ring::error::KeyRejected> for IcmtError {
    fn from(r: ring::error::KeyRejected) -> Self {
        IcmtError::RingKeyRejected(r)
    }
}
impl std::convert::From<ring::error::Unspecified> for IcmtError {
    fn from(r: ring::error::Unspecified) -> Self {
        IcmtError::RingUnspecified(r)
    }
}
impl std::convert::From<engiffen::Error> for IcmtError {
    fn from(_e: engiffen::Error) -> Self {
        IcmtError::Engiffen()
    }
}
