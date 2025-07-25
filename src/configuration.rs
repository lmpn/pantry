use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Configuration {
    pub database: DatabaseConfiguration,
    pub server: ServerConfiguration,
}

#[derive(Deserialize, Debug)]
pub struct DatabaseConfiguration {
    pub dsn: String,
}

#[derive(Deserialize, Debug)]
pub struct ServerConfiguration {
    pub host: String,
    pub port: u16,
}
