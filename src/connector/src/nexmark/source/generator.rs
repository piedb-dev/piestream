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

use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Ok, Result};

use crate::nexmark::config::NexmarkConfig;
use crate::nexmark::source::event::{Event, EventType};
use crate::nexmark::source::message::NexmarkMessage;
use crate::SourceMessage;

#[derive(Clone, Debug)]
pub struct NexmarkEventGenerator {
    pub events_so_far: u64,
    pub event_num: i64,
    pub config: Box<NexmarkConfig>,
    pub wall_clock_base_time: usize,
    pub split_index: i32,
    pub split_num: i32,
    pub split_id: String,
    pub last_event: Option<Event>,
    pub event_type: EventType,
    pub use_real_time: bool,
    pub min_event_gap_in_ns: u64,
    pub max_chunk_size: u64,
}

impl NexmarkEventGenerator {
    pub async fn next(&mut self) -> Result<Vec<SourceMessage>> {
        if self.split_num == 0 {
            return Err(anyhow::Error::msg(
                "NexmarkEventGenerator is not ready".to_string(),
            ));
        }
        let mut res: Vec<SourceMessage> = vec![];
        let mut num_event = 0;
        let old_events_so_far = self.events_so_far;

        // Get unix timestamp in milliseconds
        let current_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        if let Some(event) = &self.last_event {
            num_event += 1;
            res.push(SourceMessage::from(NexmarkMessage::new(
                self.split_id.clone(),
                self.events_so_far as u64,
                event.clone(),
            )));
        }
        self.last_event = None;

        while num_event < self.max_chunk_size {
            if self.event_num > 0 && self.events_so_far >= self.event_num as u64 {
                break;
            }

            let (event, new_wall_clock_base_time) = Event::new(
                self.events_so_far as usize,
                &self.config,
                self.wall_clock_base_time,
            );

            self.wall_clock_base_time = new_wall_clock_base_time;
            self.events_so_far += 1;

            if event.event_type() != self.event_type
                || self.events_so_far % self.split_num as u64 != self.split_index as u64
            {
                continue;
            }

            // When the generated timestamp is larger then current timestamp, if its the first
            // event, sleep and continue. Otherwise, directly return.
            if self.use_real_time && current_timestamp < new_wall_clock_base_time as u64 {
                tokio::time::sleep(std::time::Duration::from_millis(
                    new_wall_clock_base_time as u64 - current_timestamp,
                ))
                .await;

                self.last_event = Some(event);
                break;
            }

            num_event += 1;
            res.push(SourceMessage::from(NexmarkMessage::new(
                self.split_id.clone(),
                self.events_so_far as u64,
                event,
            )));
        }

        if !self.use_real_time {
            tokio::time::sleep(std::time::Duration::from_nanos(
                (self.events_so_far - old_events_so_far) as u64 * self.min_event_gap_in_ns,
            ))
            .await;
        }

        Ok(res)
    }
}
