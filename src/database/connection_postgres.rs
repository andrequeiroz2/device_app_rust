use sqlx::{postgres::PgPoolOptions, Postgres, Pool};
use log::{info, error};

struct PostgresConfig{
    max_connections: u32,
    database_url: String,
}
impl PostgresConfig{
    fn init_postgres_config() -> PostgresConfig{
        PostgresConfig{
            max_connections: std::env::var("MAX_CONNECTIONS")
                .expect("MAX_CONNECTIONS must be specified")
                .parse::<u32>()
                .expect("MAX_CONNECTIONS must be an integer number, example: \"10\""),

            database_url: std::env::var("DATABASE_URL")
                .expect("DATABASE_URL must be specified"),
        }
    }
    pub fn get_max_connections(&self) -> u32{
        self.max_connections
    }

    pub fn get_database_url(&self) -> &str{
        &self.database_url
    }
}
pub async fn get_postgres_pool() -> Pool<Postgres> {
    let config_postgres = PostgresConfig::init_postgres_config();
    
    let pool = PgPoolOptions::new()
        .max_connections(config_postgres.get_max_connections())
        .connect(&config_postgres.get_database_url())
        .await;

    match pool {
        Ok(p) => {
            info!("🐘 Successfully connected to target Postgres server!");
            p
        }

        Err(err)=> {
            error!("💥 Failed to connect to the target Postgres server!");
            error!("💥 Error: {:?}", err);
            std::process::exit(1);
        }
    }
}