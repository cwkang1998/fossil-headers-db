use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub struct Error(anyhow::Error);

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error: {}", self.0),
        )
            .into_response()
    }
}

impl<E> From<E> for Error
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

// // Any remaining endpoints related to non-MMR functionality can be added here.
// // Example placeholder:
// pub async fn get_update_info() -> Result<Json<Update>, Error> {
//     info!("Received request for latest update");

//     // Add any relevant logic here
//     let res = Update {
//         // Fill with appropriate fields or fetch data
//     };
//     Ok(Json(res))
// }
