use diesel::{deserialize::Queryable, Selectable};
use serde::Serialize;

#[derive(Queryable, Selectable, Debug, Clone, Serialize)]
#[diesel(table_name = crate::schema::book)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Book {
    pub id: String,
    pub name: String,
}

#[derive(Queryable, Selectable, Debug, Clone, Serialize)]
#[diesel(table_name = crate::schema::post)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Post {
    pub id: String,
    pub book_id: String,
    pub name: String,
}
