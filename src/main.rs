mod models;
mod paginate;
mod schema;
mod utils;
use std::net::SocketAddr;

use axum::{routing::post, Router};
use diesel::QueryDsl;
use diesel_async::pooled_connection::{bb8, AsyncDieselConnectionManager};
use utils::DatabaseConnection;

use crate::{
    models::{Book, Post},
    paginate::Paginate,
};

#[tokio::main]
async fn main() {
    let db_url = std::env::var("DATABASE_URL").unwrap();
    let config = AsyncDieselConnectionManager::<diesel_async::AsyncMysqlConnection>::new(db_url);
    let pool = bb8::Pool::builder().build(config).await.unwrap();
    let app = Router::new()
        .route("test", post(page_test))
        .with_state(pool);
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn page_test(DatabaseConnection(mut conn): DatabaseConnection) {
    use crate::schema::book::dsl::*;
    use crate::schema::post::dsl::*;
    let _result = book
        .inner_join(post)
        .paginate(1)
        .per_page(10)
        .load_and_pages::<(Book, Post)>(&mut conn)
        .await;
}
