use axum::{
    async_trait,
    extract::{FromRef, FromRequestParts},
    http::{request::Parts, StatusCode},
};
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncMysqlConnection};

pub type Connection =
    bb8::PooledConnection<'static, AsyncDieselConnectionManager<AsyncMysqlConnection>>;
pub struct DatabaseConnection(pub Connection);

type Pool = bb8::Pool<AsyncDieselConnectionManager<AsyncMysqlConnection>>;

#[async_trait]
impl<S> FromRequestParts<S> for DatabaseConnection
where
    S: Send + Sync,
    Pool: FromRef<S>,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let pool = Pool::from_ref(state);

        let conn = pool.get_owned().await.map_err(internal_error)?;

        Ok(Self(conn))
    }
}
/// Utility function for mapping any error into a `500 Internal Server Error`
/// response.
fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
