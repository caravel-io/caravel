use rand::{self, thread_rng, Rng};
use tokio::time::Duration;

/// Generate a random duration based on the interval and splay
/// The duration will be between interval - splay and interval + splay
/// If the interval is 0, it will default to 60 seconds
pub fn generate_duration(interval: u64, splay: u64) -> Duration {
    let random_splay: i64 = thread_rng().gen_range(0 - splay as i64..=splay as i64);
    let interval_seconds = (interval * 60) as i64;
    let duration = Duration::from_secs((interval_seconds + random_splay) as u64);
    // just in case they set some wonky values for interval and splay
    if duration.as_secs() <= 0 {
        return Duration::from_secs(60);
    }
    duration
}

/// Generate a simple token to be used with the /upload handler
pub fn generate_token() -> String {
    "spiderman".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_duration() {
        let interval = 30; // 30 minutes
        let splay = 10; // 10 seconds
                        // it should be between 29:50 and 30:10 or 1790 and 1810 seconds
        let d = generate_duration(interval, splay);
        println!("Duration: {:?}", d);
        assert!(d.as_secs() >= 1790 && d.as_secs() <= 1810);
    }
}
