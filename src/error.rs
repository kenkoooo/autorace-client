#[derive(Debug)]
pub enum Error {
    HtmlParseError,
    HttpIOError,
    CookieError,
}

pub type Result<T> = ::std::result::Result<T, Error>;

impl From<reqwest::Error> for Error {
    fn from(_: reqwest::Error) -> Self {
        Error::HttpIOError
    }
}
