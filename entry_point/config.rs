use anyhow::{bail, Result};



pub struct IndexerProcessorConfig {
    pub processor_config: PorcessorConfig,
    pub transaction_stream_config:  TransactionStreamConfig,
    pub db_config: DbConfig,
}


Impl IndexerProcessorConfig {
    fn validate(&self) -> Result {

    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PorcessorConfig {
    #[serde(rename = "type")]
    pub processor_type: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DbConfig {

}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct TransactionStreamConfig {

}
