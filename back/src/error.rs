pub enum ApiError {
    BadRequest(&'static str),
    Forbidden(&'static str),
    Internal(anyhow::Error),
}


// from https://claude.ai/share/31ffaae3-fc58-44d1-9165-352f2bd53d29
// #[derive(Debug, thiserror::Error)]
// pub enum AppError {
//     #[error("not found")]
//     NotFound,
//     #[error("forbidden")]
//     Forbidden,
//     #[error(transparent)]
//     Db(#[from] sqlx::Error),
//     #[error(transparent)]
//     Internal(#[from] anyhow::Error),
// }

// impl IntoResponse for AppError {
//     fn into_response(self) -> Response {
//         let status = match &self {
//             AppError::NotFound => StatusCode::NOT_FOUND,
//             AppError::Forbidden => StatusCode::FORBIDDEN,
//             _ => StatusCode::INTERNAL_SERVER_ERROR,
//         };
//         (status, self.to_string()).into_response()
//     }
// }