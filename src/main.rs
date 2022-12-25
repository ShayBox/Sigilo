use crate::entity::{player, server};
use craftping::{tokio::ping, Response};
use dotenvy_macro::dotenv;
use eyre::Result;
use futures::{stream, StreamExt};
use sea_orm::{
    sea_query::{tests_cfg::json, OnConflict},
    ActiveValue::Set,
    ConnectionTrait,
    Database,
    DatabaseBackend,
    DatabaseConnection,
    EntityTrait,
    QueryTrait,
};
use std::{
    fs::File,
    io::{BufRead, BufReader},
    time::Duration,
};
use tokio::{net::TcpStream, time::timeout};

mod entity;

#[tokio::main]
async fn main() -> Result<()> {
    let db = Database::connect(dotenv!("DATABASE_URL")).await?;

    let file = File::open("masscan.txt")?;
    let reader = BufReader::new(file);

    stream::iter(reader.lines())
        .for_each_concurrent(500, |line| async {
            if (timeout(Duration::from_secs(15), map_line(&db, line)).await).is_err() {
                println!("Did not receive response within 15 seconds")
            }
        })
        .await;

    Ok(())
}

async fn map_line(db: &DatabaseConnection, line: Result<String, std::io::Error>) -> Result<()> {
    let line = line?;
    if line.starts_with('#') {
        return Ok(());
    }

    let sections = line.split(' ').collect::<Vec<_>>();
    let port = sections[2].parse::<u16>()?;
    let host = sections[3];

    println!("Trying to ping {host}:{port}...");
    if let Ok(response) = try_ping(host, port).await {
        match handle_response(db, host, port, response).await {
            Ok(_) => println!("Successfully saved {host}:{port}"),
            Err(err) => println!("Failed to save {host}:{port}\n{err}"),
        }
    }

    Ok(())
}

async fn try_ping(host: &str, port: u16) -> Result<Response> {
    let mut stream = TcpStream::connect((host, port)).await?;
    let response = ping(&mut stream, host, port).await?;

    Ok(response)
}

async fn handle_response(
    db: &DatabaseConnection,
    host: &str,
    port: u16,
    response: Response,
) -> Result<()> {
    let address = format!("{host}:{port}");
    println!("Found server at {address}");
    println!("{response:#?}");

    let server = server::ActiveModel {
        address: Set(address),
        host: Set(host.into()),
        port: Set(port),
        response: Set(json!(response)),
        version: Set(response.version),
        protocol: Set(response.protocol),
        max_players: Set(response.max_players as u64),
        online_players: Set(response.online_players as u64),
    };

    let statement = server::Entity::insert(server)
        .on_conflict(
            OnConflict::column(server::Column::Host)
                .update_column(server::Column::Host)
                .to_owned(),
        )
        .build(DatabaseBackend::MySql);

    db.execute(statement).await?;

    let Some(samples) = response.sample else {
        return Ok(())
    };

    for sample in samples {
        let player = player::ActiveModel {
            id: Set(sample.id),
            name: Set(sample.name),
        };

        let statement = player::Entity::insert(player)
            .on_conflict(
                OnConflict::column(player::Column::Id)
                    .update_column(player::Column::Id)
                    .to_owned(),
            )
            .build(DatabaseBackend::MySql);

        db.execute(statement).await?;
    }

    Ok(())
}