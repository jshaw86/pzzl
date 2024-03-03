use std::error::Error;
use tokio;
use tokio_postgres::NoTls;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("migrations");
}
#[tokio::main]
async fn main() {
    let connection_str = "postgresql://postgres:mysecretpassword@localhost:5432?connect_timeout=10";
    let _ = run_migrations(connection_str.to_string()).await.expect("migration failed");

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
