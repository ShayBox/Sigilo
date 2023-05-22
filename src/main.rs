use std::{
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    net::Ipv4Addr,
    time::Duration,
};

use craftping::{tokio::ping, Response};
use dotenvy_macro::dotenv;
use eyre::Result;
use futures::{stream, StreamExt};
use sea_orm::{
    sea_query::{tests_cfg::json, OnConflict},
    ActiveValue::Set,
    ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, EntityTrait, QueryTrait,
};
use tokio::{net::TcpStream, time::timeout};

use crate::entity::{player, server};

mod entity;

#[tokio::main]
async fn main() -> Result<()> {
    let db = Database::connect(dotenv!("DATABASE_URL")).await?;

    create_mat1_ips_file().await?;
    for file in [File::open("mat-1.txt")?, File::open("masscan.txt")?] {
        let reader = BufReader::new(file);
        stream::iter(reader.lines())
            .for_each_concurrent(1_000, |line| async {
                if (timeout(Duration::from_secs(60), map_line(&db, line)).await).is_err() {
                    println!("Did not receive response within 60 seconds")
                }
            })
            .await;
    }

    Ok(())
}

// Generated using ChatGPT
async fn create_mat1_ips_file() -> Result<()> {
    // Remove and Create sigilo.txt
    let _ = fs::remove_file("mat-1.txt");
    let mut sigilo_file = File::create("mat-1.txt")?;

    // Download the binary file
    let url = "https://github.com/mat-1/minecraft-scans/raw/main/ips";
    let resp = reqwest::get(url).await?;
    let bytes = resp.bytes().await?;
    let buffer = bytes.to_vec();

    // Split the binary file into 6 byte chunks
    let chunks = buffer.chunks(6);
    for chunk in chunks {
        // Split the 6 byte chunks into 4 and 2 bytes
        let addr_bytes = &chunk[0..4];
        let port_bytes = &chunk[4..6];

        // Parse the bytes into an IP address and port
        let ip = Ipv4Addr::from(u32::from_be_bytes([
            addr_bytes[0],
            addr_bytes[1],
            addr_bytes[2],
            addr_bytes[3],
        ]));
        let port = u16::from_be_bytes([port_bytes[0], port_bytes[1]]);

        // Write the IP address and port to the file
        writeln!(sigilo_file, "open tcp {port} {ip} 0")?;
    }

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
        return Ok(());
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
