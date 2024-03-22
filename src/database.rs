use sqlx::{    
    postgres:: PgPoolOptions,
    PgPool,
    types::JsonValue
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum ETipoTransacao {
    #[serde(rename = "c")]
    Credito = 1,
    #[serde(rename = "d")]
    Debito = 2,
}

#[derive(Deserialize)]
pub struct CriarTransacao {
    pub valor: i64,
    pub tipo: ETipoTransacao,
    pub descricao: String,
}

impl CriarTransacao {
    pub fn eh_valido(&self) -> bool {
        !self.descricao.is_empty() && self.descricao.len() <= 10
    }
}

#[derive(Serialize, sqlx::FromRow)]
pub struct TransacaoCriada {
    #[serde(rename = "limite")]
    pub limite_resp: i32,
    #[serde(rename = "saldo")]
    pub saldo_resp: i32,
    #[serde(skip_serializing)]
    pub cliente_id_resp: i32,
    #[serde(skip_serializing)]
    pub transacao_foi_criada: bool,
}

#[derive(sqlx::FromRow)]
pub struct Extrato {
    pub id: i32,
    pub json: JsonValue,
}

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

        let _ = sqlx::query("refresh materialized view concurrently public.vw_extrato;")
            .fetch_optional(&self.pool)
            .await
            .unwrap();

        sqlx::query_as::<_, Extrato>(
            "            
            select id, extrato as \"json\" from public.vw_extrato where id = $1;
            ",
        )
        .bind(cliente_id)
        .fetch_optional(&self.pool)
        .await
        .unwrap()
    }

    pub async fn criar_transacao(
        &self,
        cliente_id: i32,
        transacao: CriarTransacao,
    ) -> TransacaoCriada {
        sqlx::query_as::<_, TransacaoCriada>("call public.criar_transacao($1::integer, $2::integer, $3::integer, $4::varchar(10));")
            .bind(cliente_id)
            .bind(transacao.valor)
            .bind(transacao.tipo as i32)
            .bind(transacao.descricao)
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }
}
