use sqlx::{
    error, postgres::{PgListener, PgPoolOptions}, types::JsonValue, PgPool
};

pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn connect(url: &str, pool_size: u32) -> Result<Self, sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(pool_size)
            .connect(url)
            .await?;

        Ok(Database { pool })
    }

    pub async fn obter_extrato(&self, cliente_id: i32) -> Option<Extrato> {        
        
        let result = sqlx::query_as::<_, Extrato>(
            "
            select id, extrato as \"json\" from public.vw_extrato where id = $1
            "
        )
        .bind(cliente_id)
        .fetch_optional(&self.pool)
        .await
        .unwrap();

        match result {
           Some(result) => Some(result),
           None => None
        }
    }
}

#[derive(sqlx::FromRow)]
pub struct Extrato {
    pub id: i32,
    pub json: JsonValue,
}
