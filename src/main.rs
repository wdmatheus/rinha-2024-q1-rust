use std::{env, sync::Arc};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use database::{CriarTransacao, Database, TransacaoCriada};

mod database;

type AppState = Arc<Database>;

fn resolve_database_url() -> String {
    env::var("DATABASE_URL").unwrap_or(String::from(
        "postgresql://postgres:123456@localhost:5432/rinha",
    ))
}

async fn connect_to_database(database_url: String) -> Database {
    let pool_size = env::var("DATABASE_POOL")
        .ok()
        .and_then(|port| port.parse::<u32>().ok())
        .unwrap_or(50);
    Database::connect(&database_url, pool_size).await.unwrap()
}

fn resolve_server_url() -> String {
    let port = env::var("PORT").ok().unwrap_or(String::from("8080"));
    format!("0.0.0.0:{port}")
}

#[tokio::main]
async fn main() {
    println!("initalized");

    let database_url = resolve_database_url();

    let database = connect_to_database(database_url).await;

    let app_state = Arc::new(database);

    let server_url = resolve_server_url();

    println!("{}", server_url);

    let app = Router::new()
        .route("/clientes/:id/extrato", get(extrato))
        .route("/clientes/:id/transacoes", post(criar_transacao))
        .with_state(app_state);

    match tokio::net::TcpListener::bind(server_url).await {
        Ok(listener) => {
            axum::serve(listener, app.into_make_service())
                .await
                .unwrap_or_else(|_| println!("error"));
        }
        Err(e) => println!("{}", e),
    }
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
                TransacaoCriada {
                    cliente_id_resp: 0,
                    limite_resp: _,
                    saldo_resp: _,
                    transacao_foi_criada: _,
                } => Err(StatusCode::NOT_FOUND),
                TransacaoCriada {
                    cliente_id_resp: _,
                    limite_resp: _,
                    saldo_resp: _,
                    transacao_foi_criada: false,
                } => Err(StatusCode::UNPROCESSABLE_ENTITY),
                _ => Ok(Json(transacao_criada)),
            }
        }
        _ => Err(StatusCode::UNPROCESSABLE_ENTITY),
    }
}
