use anyhow::Context;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::io::StdoutLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message<MessageType> {
    pub src: String,
    #[serde(rename = "dest")]
    pub dst: String,
    pub body: Body<MessageType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Body<MessageType> {
    #[serde(rename = "msg_id")]
    pub id: usize,
    pub in_reply_to: usize,
    #[serde(flatten)]
    pub msg_type: MessageType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Init {
    pub node_id: String,
    pub node_ids: Vec<String>,
}

pub trait Node<MessageType> {
    fn step(&mut self, input: Message<MessageType>, output: &mut StdoutLock) -> anyhow::Result<()>;
}

pub fn main_loop<S, MessageType>(mut state: S) -> anyhow::Result<()>
where
    S: Node<MessageType>,
    MessageType: DeserializeOwned,
{
    let stdin = std::io::stdin().lock();
    let inputs = serde_json::Deserializer::from_reader(stdin).into_iter::<Message<MessageType>>();

    let mut stdout = std::io::stdout().lock();

    for input in inputs {
        let input = input.context("Maelstrom input from STDIN could not be deserialized")?;
        state
            .step(input, &mut stdout)
            .context("Node step function failed")?;
    }

    Ok(())
}
