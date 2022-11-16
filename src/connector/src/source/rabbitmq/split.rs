use serde::{Deserialize, Serialize};
use crate::source::{SplitId, SplitMetaData};
use anyhow::anyhow;
use bytes::Bytes;


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Hash)]
pub struct RabbitmqSplit {
    pub(crate) topic: String,
}

impl SplitMetaData for RabbitmqSplit {
    fn id(&self) -> SplitId {
        // TODO: should avoid constructing a string every time
        self.topic.to_string().into()
    }

    fn encode_to_bytes(&self) -> Bytes {
        Bytes::from(serde_json::to_string(self).unwrap())
    }

    fn restore_from_bytes(bytes: &[u8]) -> anyhow::Result<Self> {
        serde_json::from_slice(bytes).map_err(|e| anyhow!(e))
    }
}
