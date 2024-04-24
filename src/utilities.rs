use std::time::{Duration, SystemTime, SystemTimeError};

use md5::{Digest, Md5};
use rand::Rng;

/// converts provided plaintext to md5 hash
pub fn md5_hash(plaintext: &str) -> String {
    let mut hasher = Md5::new();
    hasher.update(plaintext.as_bytes());
    let result = hasher.finalize();

    hex::encode(result)
}

/// generates random timestamp within this week
pub fn random_timestamp() -> Result<u64, SystemTimeError> {
    let current_date = SystemTime::now();

    let start_of_week = {
        let days_since_sunday = current_date
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs()
            % (7 * 24 * 3600);
        current_date - Duration::from_secs(days_since_sunday)
    };

    let end_of_week = start_of_week + Duration::from_secs(6 * 24 * 3600);

    let mut rng = rand::thread_rng();
    let mut random_timestamp;

    loop {
        let random_duration =
            rng.gen_range(0..=(end_of_week.duration_since(start_of_week)?.as_secs()));
        random_timestamp = start_of_week + Duration::from_secs(random_duration);

        if random_timestamp <= current_date {
            break;
        }
    }

    let timestamp = random_timestamp
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs();
    Ok(timestamp)
}
