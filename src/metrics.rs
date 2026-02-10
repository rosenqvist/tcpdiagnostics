#[derive(Debug, Clone)]
pub struct RttMetrics {
    pub sent: u32,
    pub received: u32,
    pub min_ms: f64,
    pub avg_ms: f64,
    pub max_ms: f64,
}

pub fn compute_rtt_metrics(samples_ms: &[f64], sent: u32) -> RttMetrics {
    let received = samples_ms.len() as u32;

    if received == 0 {
        return RttMetrics {
            sent,
            received,
            min_ms: 0.0,
            avg_ms: 0.0,
            max_ms: 0.0,
        };
    }

    let mut min = samples_ms[0];
    let mut max = samples_ms[0];
    let mut sum = 0.0;

    for v in samples_ms {
        if *v < min {
            min = *v;
        }
        if *v > max {
            max = *v;
        }
        sum += *v;
    }

    let avg = sum / samples_ms.len() as f64;

    RttMetrics {
        sent,
        received,
        min_ms: min,
        avg_ms: avg,
        max_ms: max,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_basic() {
        let samples = vec![10.0, 20.0, 30.0];
        let metrics = compute_rtt_metrics(&samples, 3);

        assert_eq!(metrics.sent, 3);
        assert_eq!(metrics.received, 3);
        assert_eq!(metrics.min_ms, 10.0);
        assert_eq!(metrics.max_ms, 30.0);
        assert_eq!(metrics.avg_ms, 20.0);
    }
}
