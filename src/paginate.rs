use diesel::{
    dsl::{count_star, CountStar},
    mysql::Mysql,
    query_builder::{AsQuery, Query, QueryFragment, QueryId},
    query_dsl::methods::{LimitDsl, OffsetDsl, SelectDsl},
    QueryResult,
};
use diesel_async::{methods::LoadQuery, AsyncMysqlConnection, RunQueryDsl};

use crate::utils::Connection;

#[derive(Clone, Debug)]
pub struct Page<T> {
    pub page: i64,
    pub total: i64,
    pub total_page: i64,
    pub per_page: i64,
    pub data: Vec<T>,
}

pub trait Paginate: Sized + AsQuery {
    fn paginate(self, page: i64) -> Paginated<Self::Query> {
        Paginated {
            query: self.as_query(),
            page,
            per_page: DEFAULT_PER_PAGE,
            offset: (page - 1) * DEFAULT_PER_PAGE,
        }
    }
}

const DEFAULT_PER_PAGE: i64 = 10;

impl<T: AsQuery> Paginate for T {}

impl<T> QueryFragment<Mysql> for Paginated<T>
where
    T: QueryFragment<Mysql>,
{
    fn walk_ast<'b>(
        &'b self,
        mut out: diesel::query_builder::AstPass<'_, 'b, Mysql>,
    ) -> diesel::prelude::QueryResult<()> {
        self.query.walk_ast(out.reborrow())?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, QueryId)]
pub struct Paginated<T> {
    query: T,
    page: i64,
    per_page: i64,
    offset: i64,
}

impl<T> Paginated<T> {
    pub fn per_page(self, per_page: i64) -> Self {
        Paginated {
            per_page,
            offset: (self.page - 1) * per_page,
            ..self
        }
    }

    pub fn load_and_pages<'query, 'conn, U>(
        self,
        conn: &'query mut Connection,
    ) -> impl std::future::Future<Output = QueryResult<Page<U>>> + Send + 'query
    where
        T: LimitDsl + SelectDsl<CountStar> + Clone,
        <T as LimitDsl>::Output: OffsetDsl,
        <T as SelectDsl<CountStar>>::Output: LimitDsl + QueryFragment<Mysql>,
        <<T as SelectDsl<CountStar>>::Output as LimitDsl>::Output: Send + 'query,
        <<T as LimitDsl>::Output as OffsetDsl>::Output:
            QueryFragment<Mysql> + LoadQuery<'query, AsyncMysqlConnection, U> + 'query,
        <<T as SelectDsl<CountStar>>::Output as LimitDsl>::Output:
            LoadQuery<'query, AsyncMysqlConnection, i64>,
        U: 'query + Send + Sized,
    {
        let query = self.query.clone().limit(self.per_page).offset(self.offset);
        let records = query.load::<U>(conn);
        let query = self.query.select(count_star());
        let total = query.first::<i64>(conn);
        async move {
            let records = records.await?;
            let total = total.await?;
            let total_page = (total as u64).div_ceil(self.per_page as u64) as i64;
            let page = Page {
                total,
                total_page,
                page: self.page,
                per_page: self.page,
                data: records,
            };
            Ok(page)
        }
    }
}

impl<T: Query> Query for Paginated<T> {
    type SqlType = T::SqlType;
}
