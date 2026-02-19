mod client;
mod metrics;
mod protocol;
mod report;
mod server;

#[cfg(test)]
mod tests;

use std::process;

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    match args[1].as_str() {
        "server" => cmd_server(&args),
        "client" => cmd_client(&args),
        "diagnose" => cmd_diagnose(&args),
        "-h" | "--help" | "help" => {
            print_usage();
            Ok(())
        }
        _ => {
            print_usage();
            Ok(())
        }
    }
}

fn cmd_server(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    // tcpdiagnostics server <bind_addr>
    if args.len() != 3 {
        return Err("usage: tcpdiagnostics server <bind_addr>".into());
    }

    server::run_server(&args[2])?;
    Ok(())
}

fn cmd_client(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    // Keeping legacy form for now:
    // tcpdiagnostics client <target_addr> rtt <count>
    if args.len() == 5 && args[3] == "rtt" {
        let target = &args[2];
        let count = parse_u32(&args[4], "count")?;

        let m = client::run_rtt(target, count)?;

        let report = report::DiagnosticReport {
            target: target.to_string(),
            connect_ms: None,
            rtt: Some(m),
            warnings: Vec::new(),
        };

        report.print_human();
        return Ok(());
    }

    Err("usage: tcpdiagnostics client <target_addr> rtt <count>".into())
}

fn cmd_diagnose(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    // tcpdiagnostics diagnose <target_addr> [count] [--json]
    if args.len() < 3 {
        return Err("usage: tcpdiagnostics diagnose <target_addr> [count] [--json]".into());
    }

    let target = &args[2];

    let mut count: u32 = 50;
    let mut json = false;

    for a in &args[3..] {
        if a == "--json" {
            json = true;
            continue;
        }
        count = parse_u32(a, "count")?;
    }

    let report = client::run_diagnose(target, count)?;

    if json {
        println!("{}", report.to_json_pretty());
    } else {
        report.print_human();
    }

    Ok(())
}

fn parse_u32(s: &str, field: &str) -> Result<u32, Box<dyn std::error::Error>> {
    s.parse::<u32>()
        .map_err(|_| format!("invalid {field} '{s}', expected a u32").into())
}

fn print_usage() {
    eprintln!("usage:");
    eprintln!("  tcpdiagnostics server <bind_addr>");
    eprintln!("  tcpdiagnostics client <target_addr> rtt <count>");
    eprintln!("  tcpdiagnostics diagnose <target_addr> [count] [--json]");
}
