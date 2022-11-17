use serde::{Deserialize, Serialize};
use crate::source::{SplitId, SplitMetaData};
use anyhow::anyhow;
use bytes::Bytes;


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct RabbitmqSplit {
    pub(crate) queue_name: String,
}


impl RabbitmqSplit {
    pub fn copy_with_offset(&self, start_offset: String) -> Self {
        Self {
            queue_name: "hello".to_string(),
            // start_offset,
        }
    }
}


impl SplitMetaData for RabbitmqSplit {
    fn id(&self) -> SplitId {
        // TODO: should avoid constructing a string every time
        self.queue_name.to_string().into()
    }

    fn encode_to_bytes(&self) -> Bytes {
        Bytes::from(serde_json::to_string(self).unwrap())
    }

    fn restore_from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        serde_json::from_slice(bytes).map_err(|e| anyhow!(e))
    }
}
