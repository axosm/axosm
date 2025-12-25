pub enum ApiError {
    BadRequest(&'static str),
    Forbidden(&'static str),
    Internal(anyhow::Error),
}