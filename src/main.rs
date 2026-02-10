mod client;
mod metrics;
mod protocol;
mod server;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 3 && args[1] == "server" {
        server::run_server(&args[2]).unwrap();
        return;
    }

    if args.len() == 5 && args[1] == "client" && args[3] == "rtt" {
        let target = &args[2];
        let count: u32 = args[4].parse().unwrap();

        let m = client::run_rtt(target, count).unwrap();

        println!("sent:     {}", m.sent);
        println!("received: {}", m.received);
        println!("min_ms:   {:.3}", m.min_ms);
        println!("avg_ms:   {:.3}", m.avg_ms);
        println!("max_ms:   {:.3}", m.max_ms);
        return;
    }

    eprintln!("usage:");
    eprintln!("  tcpdiagnostics server <bind_addr>");
    eprintln!("  tcpdiagnostics client <target_addr> rtt <count>");
}

#[cfg(test)]
mod tests;
