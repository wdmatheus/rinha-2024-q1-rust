use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum ETipoTransacao {
    #[serde(rename = "c")]
    Credito,
    #[serde(rename = "d")]
    Debito,
}

#[derive(Deserialize)]
pub struct CriarTransacao {
    pub valor: i64,
    pub tipo: ETipoTransacao,
    pub descricao: String,
}

impl CriarTransacao {
    pub fn eh_valido(&self) -> bool {
        return !self.descricao.is_empty() && self.descricao.len() <= 10;
    }
}

#[derive(Serialize)]
pub struct TransacaoCriada {
    pub limite: i64,
    pub saldo: i64,
}
