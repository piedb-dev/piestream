


use pulsar::consumer::Message;

use crate::source::SourceMessage;

// impl From<Message<Vec<u8>>> for SourceMessage {
//     fn from(msg: Message<Vec<u8>>) -> Self {
//         let message_id = msg.message_id.id;

//         SourceMessage {
//             payload: Some(msg.payload.data.into()),
//             offset: format!(
//                 "{}:{}:{}:{}",
//                 message_id.ledger_id,
//                 message_id.entry_id,
//                 message_id.partition.unwrap_or(-1),
//                 message_id.batch_index.unwrap_or(-1)
//             ),
//             split_id: msg.topic.into(),
//         }
//     }
// }

