use std::error::Error;
use tokio;
use tokio_postgres::NoTls;
use clap::Parser;


mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("migrations");
}

#[derive(Debug, Parser)]
pub struct Config {
    #[clap(long, env)]
    database_url: String,
    #[clap(long, env)]
    database_user: String,
    #[clap(long, env)]
    database_password: String,
    #[clap(long, env)]
    database_timeout: u8,
}

#[tokio::main]
async fn main() {
    let conf = Config::parse();

    let db_connection_str = format!("postgresql://{user}:{password}@{url}/?connect_timeout={timeout}", 
                                    user=conf.database_user, password=conf.database_password, url=conf.database_url, timeout=conf.database_timeout);
    let _ = run_migrations(db_connection_str.to_string()).await.expect("migration failed");

}


async fn run_migrations(connection_str: String) -> std::result::Result<(), Box<dyn Error>> {
    println!("Running DB migrations...");
    let (mut client, con) = tokio_postgres::connect(&connection_str, NoTls).await?;

    tokio::spawn(async move {
        if let Err(e) = con.await {
            eprintln!("connection error: {}", e);
        }
    });

    let migration_report = embedded::migrations::runner()
        .run_async(&mut client)
        .await?;

    for migration in migration_report.applied_migrations() {
        println!(
            "Migration Applied -  Name: {}, Version: {}",
            migration.name(),
            migration.version()
        );
    }

    println!("DB migrations finished!");

    Ok(())
}
