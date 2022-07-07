// Copyright 2022 Singularity Data
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use core::fmt;
use std::cmp::Ordering;
use std::time::{Duration, SystemTime};

lazy_static::lazy_static! {
    /// `UNIX_SINGULARITY_DATE_EPOCH` represents the singularity date of the UNIX epoch: 2021-04-01T00:00:00Z.
    pub static ref UNIX_SINGULARITY_DATE_EPOCH: SystemTime = SystemTime::UNIX_EPOCH + Duration::from_secs(1_617_235_200);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Epoch(pub u64);

/// `INVALID_EPOCH` defines the invalid epoch value.
pub const INVALID_EPOCH: u64 = 0;

const EPOCH_PHYSICAL_SHIFT_BITS: u8 = 16;

impl Epoch {
    pub fn now() -> Self {
        Self(Self::physical_now() << EPOCH_PHYSICAL_SHIFT_BITS)
    }

    #[must_use]
    pub fn next(self) -> Self {
        let physical_now = Epoch::physical_now();
        let prev_physical_time = self.physical_time();
        match physical_now.cmp(&prev_physical_time) {
            Ordering::Greater => Epoch(physical_now << EPOCH_PHYSICAL_SHIFT_BITS),
            Ordering::Equal => {
                tracing::warn!("New generate epoch is too close to the previous one.");
                Epoch(self.0 + 1)
            }
            Ordering::Less => {
                tracing::warn!(
                    "Clock goes backwards when calling Epoch::next(): prev={}, curr={}",
                    prev_physical_time,
                    physical_now
                );
                Epoch(self.0 + 1)
            }
        }
    }

    pub fn physical_time(&self) -> u64 {
        self.0 >> EPOCH_PHYSICAL_SHIFT_BITS
    }

    fn physical_now() -> u64 {
        UNIX_SINGULARITY_DATE_EPOCH
            .elapsed()
            .expect("system clock set earlier than singularity date!")
            .as_millis() as u64
    }

    /// Returns the epoch in real system time.
    pub fn as_system_time(&self) -> SystemTime {
        *UNIX_SINGULARITY_DATE_EPOCH + Duration::from_millis(self.physical_time())
    }

    /// Returns the epoch subtract `relative_time_ms`, which used for ttl to get epoch corresponding
    /// to the lowerbound timepoint (`src/storage/src/hummock/iterator/forward_user.rs`)
    pub fn subtract_ms(&self, relative_time_ms: u64) -> Self {
        let physical_time = self.physical_time();

        if physical_time < relative_time_ms {
            Epoch(INVALID_EPOCH)
        } else {
            Epoch((physical_time - relative_time_ms) << EPOCH_PHYSICAL_SHIFT_BITS)
        }
    }
}

impl From<u64> for Epoch {
    fn from(epoch: u64) -> Self {
        Self(epoch)
    }
}

impl fmt::Display for Epoch {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Local, TimeZone, Utc};

    use super::*;

    #[test]
    fn test_singularity_system_time() {
        let utc = Utc.ymd(2021, 4, 1).and_hms(0, 0, 0);
        let singularity_dt = Local.from_utc_datetime(&utc.naive_utc());
        let singularity_st = SystemTime::from(singularity_dt);
        assert_eq!(singularity_st, *UNIX_SINGULARITY_DATE_EPOCH);
    }

    #[test]
    fn test_epoch_generate() {
        let mut prev_epoch = Epoch::now();
        for _ in 0..1000 {
            let epoch = prev_epoch.next();
            assert!(epoch > prev_epoch);
            prev_epoch = epoch;
        }
    }

    #[test]
    fn test_subtract_ms() {
        {
            let epoch = Epoch(10);
            assert_eq!(0, epoch.physical_time());
            assert_eq!(0, epoch.subtract_ms(20).0);
        }

        {
            let epoch = Epoch::now();
            let physical_time = epoch.physical_time();
            let interval = 10;

            assert_ne!(0, physical_time);
            assert_eq!(
                physical_time - interval,
                epoch.subtract_ms(interval).physical_time()
            );
        }
    }
}
