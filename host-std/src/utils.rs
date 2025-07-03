//! Utils functions for the host device

use esp_idf_sys::random;
use uuid::Uuid;

/// Get a UUID from a random number generator
pub unsafe fn random_uuid() -> Uuid {
    let mut res: u128 = 0;
    for _ in 0..(128 / 32) {
        res = res << 32 | random() as u128;
    }
    Uuid::from_u128(res)
}
