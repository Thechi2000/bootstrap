use crate::updater::Message;

#[derive(Debug)]
/// Represents all errors that may occur
pub enum Error{
    IO(std::io::Error),
    Path(std::path::StripPrefixError),
    Hash(openssl::error::ErrorStack),
    Url(url::ParseError),
    Utf8(std::str::Utf8Error),
    Json(serde_json::Error),
    Reqwest(reqwest::Error),
    MpscSend(tokio::sync::mpsc::error::SendError<Message>),
    Other(String),
}

impl From<String> for Error{
    fn from(err: String)-> Self{
        Error::Other(err)
    }
}

impl From<tokio::sync::mpsc::error::SendError<Message>> for Error{
    fn from(err: tokio::sync::mpsc::error::SendError<Message>)-> Self{
        Error::MpscSend(err)
    }
}

impl From<serde_json::Error> for Error{
    fn from(err: serde_json::Error)-> Self{
        Error::Json(err)
    }
}

impl From<reqwest::Error> for Error{
    fn from(err: reqwest::Error)-> Self{
        Error::Reqwest(err)
    }
}

impl From<std::str::Utf8Error> for Error{
    fn from(err: std::str::Utf8Error)-> Self{
        Error::Utf8(err)
    }
}

impl From<url::ParseError> for Error{
    fn from(err: url::ParseError)-> Self{
        Error::Url(err)
    }
}

impl From<std::io::Error> for Error{
    fn from(err: std::io::Error)-> Self{
        Error::IO(err)
    }
}

impl From<openssl::error::ErrorStack> for Error{
    fn from(err: openssl::error::ErrorStack) -> Self{
        Error::Hash(err)
    }
}

impl From<std::path::StripPrefixError> for Error{
    fn from(err: std::path::StripPrefixError) -> Self{
        Error::Path(err)
    }
}