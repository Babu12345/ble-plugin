// Copyright 2025 Wanyeki Technologies LLC. All rights reserved.
//
// This source code is proprietary and confidential. Unauthorized copying,
// modification, distribution, or use of this software is strictly prohibited.

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
