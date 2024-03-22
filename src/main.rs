use std::{env, sync::Arc};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use database::{Database, CriarTransacao, TransacaoCriada};

mod database;

type AppState = Arc<Database>;

#[tokio::main]
async fn main() {
    let database_url = env::var("DATABASE_URL").unwrap_or(String::from(
        "postgresql://postgres:123456@localhost:5432/rinha",
    ));

    let pool_size = env::var("DATABASE_POOL")
        .ok()
        .and_then(|port| port.parse::<u32>().ok())
        .unwrap_or(30);

    let database = Database::connect(&database_url, pool_size).await.unwrap();

    let app_state = Arc::new(database);

    let port = env::var("PORT").ok().unwrap_or(String::from("9999"));

    let address = format!("0.0.0.0:{port}");

    let app = Router::new()
        .route("/clientes/:id/extrato", get(extrato))
        .route("/clientes/:id/transacoes", post(criar_transacao))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(address).await.unwrap();

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

async fn extrato(State(database): State<AppState>, Path(id): Path<u8>) -> impl IntoResponse {
    match database.obter_extrato(id as i32).await {
        Some(extrato) => Ok((
            StatusCode::OK,
            [("Content-Type", "application/json")],
            extrato.json.to_string(),
        )),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn criar_transacao(
    State(database): State<AppState>,
    Path(id): Path<u8>,
    Json(payload): Json<CriarTransacao>,
) -> impl IntoResponse {
    match payload.eh_valido() {
        true => {
            let transacao_criada = database.criar_transacao(id as i32, payload).await;

            match transacao_criada {
                TransacaoCriada { cliente_id_resp: 0, limite_resp:_, saldo_resp: _, transacao_foi_criada: _ } =>  Err(StatusCode::NOT_FOUND),
                TransacaoCriada { cliente_id_resp: _, limite_resp: _, saldo_resp: _, transacao_foi_criada: false } =>  Err(StatusCode::UNPROCESSABLE_ENTITY),
                _ => Ok(Json(transacao_criada))
            }
            
        }
        _ => Err(StatusCode::UNPROCESSABLE_ENTITY),
    }    
}
