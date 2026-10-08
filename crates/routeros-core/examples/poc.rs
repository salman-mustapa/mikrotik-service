//! Proof of concept: login, print, and listen.
//!
//! cargo run -p routeros-core --example poc -- 192.168.88.1:8728 admin password

use futures::StreamExt;
use routeros_core::{build_command, Client};

#[tokio::main]
async fn main() -> routeros_core::Result<()> {
    let mut args = std::env::args().skip(1);
    let addr = args.next().expect("usage: poc <host:port> <user> [password]");
    let user = args.next().expect("usage: poc <host:port> <user> [password]");
    let pass = args.next().unwrap_or_default();

    let client = Client::connect(addr, &user, &pass).await?;
    println!("logged in");

    // 1. one-shot print
    let rows = client
        .run(build_command("/system/resource/print", std::iter::empty::<(&str, &str)>()))
        .await?;
    for r in &rows {
        println!("{:#?}", r.attrs);
    }

    // 2. streaming: first 5 samples of traffic on the first interface
    let ifaces = client.run(build_command("/interface/print", [(".proplist", "name")])).await?;
    let name = ifaces.first().and_then(|r| r.get("name")).unwrap_or("ether1").to_owned();
    println!("monitoring {name}");

    let mut sub = client
        .listen(build_command("/interface/monitor-traffic", [("interface", name.as_str())]))
        .await?;
    for _ in 0..5 {
        match sub.next().await {
            Some(Ok(s)) => println!(
                "rx={} tx={}",
                s.get("rx-bits-per-second").unwrap_or("?"),
                s.get("tx-bits-per-second").unwrap_or("?")
            ),
            Some(Err(e)) => return Err(e),
            None => break,
        }
    }
    drop(sub); // sends /cancel
    Ok(())
}
