use rumqttc::{AsyncClient, Publish};
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;

pub(crate) struct Mqtt {
    pub task: JoinHandle<()>,
    pub client: AsyncClient,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Payload {
  pub id: String,
  pub event: PayloadEvent,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) enum PayloadEvent {
  #[serde(rename = "connect")]
  Connect(),
  #[serde(rename = "disconnect")]
  Disconnect(),
  #[serde(rename = "message")]
  Message(MqttPublish),
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct MqttPublish {
    pub dup: bool,
    pub qos: u8,
    pub retain: bool,
    pub topic: String,
    pub pkid: u16,
    pub payload: Vec<u8>,
}

impl From<Publish> for MqttPublish {
    fn from(p: Publish) -> Self {
        Self {
            dup: p.dup,
            qos: match p.qos {
                rumqttc::QoS::AtMostOnce => 0,
                rumqttc::QoS::AtLeastOnce => 1,
                rumqttc::QoS::ExactlyOnce => 2,
            },
            retain: p.retain,
            topic: p.topic,
            pkid: p.pkid,
            payload: p.payload.to_vec(),
        }
    }
}
