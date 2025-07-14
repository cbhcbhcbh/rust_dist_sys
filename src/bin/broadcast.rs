use rust_dist_sys::*;

use anyhow::{Context, Ok};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    io::{StdoutLock, Write},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
enum MessageType {
    Broadcast {
        message: usize,
    },
    BroadcastOk,
    Read,
    ReadOk {
        messages: Vec<usize>,
    },
    Topology {
        topology: HashMap<String, Vec<String>>,
    },
    TopologyOk,
}

struct BroadcastNode {
    node: String,
    id: usize,
    messages: Vec<usize>,
}

impl Node<(), MessageType> for BroadcastNode {
    fn from_init(_state: (), init: Init) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Ok(BroadcastNode {
            node: init.node_id,
            id: 1,
            messages: Vec::new(),
        })
    }

    fn step(&mut self, input: Message<MessageType>, output: &mut StdoutLock) -> anyhow::Result<()> {
        let mut reply = input.into_reply(Some(&mut self.id));
        match reply.body.msg_type {
            MessageType::Broadcast { message } => {
                self.messages.push(message);
                reply.body.msg_type = MessageType::BroadcastOk;
                serde_json::to_writer(&mut *output, &reply)
                    .context("serialize response to broadcast")?;
                output.write_all(b"\n").context("write trailing newline")?;
            }
            MessageType::Read => {
                reply.body.msg_type = MessageType::ReadOk {
                    messages: self.messages.clone(),
                };
                serde_json::to_writer(&mut *output, &reply)
                    .context("serialize response to read")?;
                output.write_all(b"\n").context("write trailing newline")?;
            }
            MessageType::Topology { topology: _ } => {
                reply.body.msg_type = MessageType::TopologyOk;
                serde_json::to_writer(&mut *output, &reply)
                    .context("serialize response to topology")?;
                output.write_all(b"\n").context("write trailing newline")?;
            }
            MessageType::BroadcastOk | MessageType::ReadOk { .. } | MessageType::TopologyOk => {}
        }
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop::<_, BroadcastNode, _>(())
}
