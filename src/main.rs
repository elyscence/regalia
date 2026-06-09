use mc_server_status::{McClient, ServerEdition, ServerStatus};
use rand::prelude::*;
use std::{
    net::{Ipv4Addr, ToSocketAddrs},
    sync::Arc,
    time::Duration,
};
use tokio::{net::TcpStream, select, sync::Semaphore, task::JoinSet, time::timeout};

#[tokio::main]
async fn main() {
    let mut set = JoinSet::new();
    let semaphore = Arc::new(Semaphore::new(5000));
    let client = Arc::new(
        McClient::new()
            .with_timeout(Duration::from_secs(5))
            .with_max_parallel(10),
    );

    loop {
        select! {
            Ok(permit) = semaphore.clone().acquire_owned() => {
                let ip = generate_random_ip().to_string();
                let client = client.clone();
                set.spawn(async move {
                    let _permit = permit;
                    check(&ip, &client).await
                });
            }
            Some(result) = set.join_next() => {
                match result {
                    Ok(Some(info)) => println!("{:?}", info),
                    Ok(None) => (),
                    Err(e) => println!("panic: {e}"),
                }
            }
        }
    }
}

async fn check(ip: &str, client: &McClient) -> Option<ServerStatus> {
    let _stream = connect_to_server_tcp(ip).await?;

    client.ping(ip, ServerEdition::Java).await.ok()
}

async fn connect_to_server_tcp(ip: &str) -> Option<TcpStream> {
    let server_ip = format!("{ip}:25565").to_socket_addrs().ok()?.next()?;

    timeout(Duration::from_secs(5), TcpStream::connect(server_ip))
        .await
        .ok()?
        .ok()
}

fn generate_random_ip() -> Ipv4Addr {
    let mut rng = rand::rng();

    loop {
        let ip = Ipv4Addr::new(
            rng.random_range(1..=239),
            rng.random_range(1..=255),
            rng.random_range(1..=255),
            rng.random_range(1..=255),
        );

        if is_public(ip) {
            return ip;
        }
    }
}

fn is_public(ip: Ipv4Addr) -> bool {
    let octets = ip.octets();
    !ip.is_private()
        && !ip.is_loopback()
        && !ip.is_multicast()
        && !ip.is_link_local()
        && !ip.is_unspecified()
        && !ip.is_broadcast()
        && !(octets[0] == 100 && octets[1] >= 64 && octets[1] <= 127)
        && !(octets[0] == 192 && octets[1] == 0 && octets[2] == 2)
        && !(octets[0] == 198 && octets[1] == 51 && octets[2] == 100)
        && !(octets[0] == 203 && octets[1] == 0 && octets[2] == 113)
}
