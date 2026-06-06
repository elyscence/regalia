use anyhow::{Result, anyhow};
use clap::Parser;
use mc_server_status::{McClient, ServerEdition};
use std::{
    net::{TcpStream, ToSocketAddrs},
    time::Duration,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    ip: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    let ip = args.ip;

    let _ = connect_to_server_tcp(&ip);
    let _ = check_minecraft_status(&ip).await;
}

fn connect_to_server_tcp(ip: &str) -> Result<()> {
    let server_ip = format!("{ip}:25565")
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| anyhow!("No address resolved for {ip}"))?;
    if let Ok(_) = TcpStream::connect_timeout(&server_ip, Duration::from_secs(1)) {
        println!("Connected to the server");
    } else {
        println!("Couldn't connect to server...");
    }
    Ok(())
}

async fn check_minecraft_status(ip: &str) -> Result<()> {
    let client = McClient::new()
        .with_timeout(Duration::from_secs(5))
        .with_max_parallel(10);

    let status = client.ping(ip, ServerEdition::Java).await?;
    println!("Status: {:?}", status);

    Ok(())
}
