mod client;
mod metrics;
mod protocol;
mod server;

use std::process;

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        process::exit(1);
    }
}

//helper function for match printing & formatting
//format! returns a string, for "None" we also return a string to keep compiler happy
fn fmt_ms(v: Option<f64>) -> String {
    match v {
        Some(x) => format!("{:.3} ms", x),
        None => "N/A".to_string(),
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 3 && args[1] == "server" {
        server::run_server(&args[2])?;
        return Ok(());
    }

    if args.len() == 5 && args[1] == "client" && args[3] == "rtt" {
        let target = &args[2];

        // give meaningful error message here
        let count: u32 = args[4]
            .parse()
            .map_err(|_| format!("invalid count '{}', expected a u32", args[4]))?;

        let m = client::run_rtt(target, count)?;

        println!("sent:     {}", m.sent);
        println!("received: {}", m.received);
        println!("min: {}", fmt_ms(m.min_ms));
        println!("avg: {}", fmt_ms(m.avg_ms));
        println!("max: {}", fmt_ms(m.max_ms));
        return Ok(());
    }

    eprintln!("usage:");
    eprintln!("  tcpdiagnostics server <bind_addr>");
    eprintln!("  tcpdiagnostics client <target_addr> rtt <count>");
    Ok(())
}

#[cfg(test)]
mod tests;
