use crate::metrics::RttMetrics;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct DiagnosticReport {
    pub target: String,
    pub connect_ms: Option<f64>,
    pub rtt: Option<RttMetrics>,
    pub warnings: Vec<String>,
}

pub fn fmt_ms(v: Option<f64>) -> String {
    match v {
        Some(x) => format!("{:.3} ms", x),
        None => "N/A".to_string(),
    }
}

impl DiagnosticReport {
    pub fn print_human(&self) {
        println!("target:   {}", self.target);
        println!("connect:  {}", fmt_ms(self.connect_ms));

        match &self.rtt {
            Some(m) => {
                println!("sent:     {}", m.sent);
                println!("received: {}", m.received);
                println!("min:      {}", fmt_ms(m.min_ms));
                println!("avg:      {}", fmt_ms(m.avg_ms));
                println!("max:      {}", fmt_ms(m.max_ms));
            }
            None => {
                println!("rtt:      N/A");
            }
        }

        if !self.warnings.is_empty() {
            println!();
            println!("warnings:");
            for w in &self.warnings {
                println!("- {}", w);
            }
        }
    }

    pub fn to_json_pretty(&self) -> String {
        serde_json::to_string_pretty(self).expect("failed to serialize DiagnosticReport")
    }

    pub fn to_json_compact(&self) -> String {
        serde_json::to_string(self).expect("failed to serialize DiagnosticReport")
    }
}
