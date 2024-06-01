use aws_lambda_events::event::eventbridge::EventBridgeEvent;
use lambda_runtime::{ tracing, Error, service_fn, LambdaEvent};
use clap::Parser;

use anyhow::Result;
use rustls::ClientConfig as RustlsClientConfig;
use serde::{Deserialize, Serialize};
use std::{fs::File, io::BufReader};
use tokio_postgres::NoTls;
use tokio_postgres_rustls::MakeRustlsConnect;


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
    #[clap(long, env)]
    db_ca_cert: Option<String>
}


#[derive(Debug, Deserialize, Serialize)]
struct TimedEvent {}

async fn service_callback(_: LambdaEvent<EventBridgeEvent<TimedEvent>>) -> Result<(), Error> {
    let conf = Config::parse();

    let database_url = format!("postgresql://{user}:{password}@{url}/?connect_timeout={timeout}", 
                                    user=conf.database_user, password=conf.database_password, url=conf.database_url, timeout=conf.database_timeout);

    let mut client = if let Some(ca_cert) = conf.db_ca_cert {
        println!("cert {}", &ca_cert);
        let cert_file = File::open(ca_cert)?;
        let mut buf = BufReader::new(cert_file);
        let mut root_store = rustls::RootCertStore::empty();
        for cert in rustls_pemfile::certs(&mut buf) {
            root_store.add(cert?)?;
        }

        let tls_config = RustlsClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();

        let tls = MakeRustlsConnect::new(tls_config);
        let (client, conn) = tokio_postgres::connect(&database_url,tls).await?;
         if let Err(e) = conn.await {
            eprintln!("connection error: {}", e);
         }

        client
    } else {
        let (client, conn) = tokio_postgres::connect(&database_url,NoTls).await?;
        if let Err(e) = conn.await {
            eprintln!("connection error: {}", e);
         }
        client
    };

    let migration_report = embedded::migrations::runner()
        .run_async(&mut client).await?;

    for migration in migration_report.applied_migrations() {
        println!(
            "Migration Applied -  Name: {}, Version: {}",
            migration.name(),
            migration.version()
        );
    }

    Ok(())
}

#[tokio::main]
async fn main()  -> Result<(), Error> {
    tracing::init_default_subscriber();
    
    lambda_runtime::run(service_fn(service_callback)).await?;
    Ok(())
   
}

