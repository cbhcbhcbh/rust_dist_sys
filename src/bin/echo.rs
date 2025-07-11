use anyhow::{Context, bail};
use serde::{Deserialize, Serialize};
use std::io::{StdoutLock, Write};

use rust_dist_sys::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
enum MessageType {
    Echo {
        echo: String,
    },
    EchoOk {
        echo: String,
    },
    Init {
        node_id: String,
        node_ids: Vec<String>,
    },
    InitOk,
}

struct EchoNode {
    id: usize,
}

impl Node<MessageType> for EchoNode {
    fn step(&mut self, input: Message<MessageType>, output: &mut StdoutLock) -> anyhow::Result<()> {
        match input.body.msg_type {
            MessageType::Init { .. } => {
                let reply = Message {
                    src: input.dst,
                    dst: input.src,
                    body: Body {
                        id: self.id,
                        in_reply_to: input.body.id,
                        msg_type: MessageType::InitOk,
                    },
                };
                serde_json::to_writer(&mut *output, &reply)
                    .context("serialize response to init")?;
                output.write_all(b"\n").context("write trailing newline")?;
                self.id += 1;
            }
            MessageType::Echo { echo } => {
                let reply = Message {
                    src: input.dst,
                    dst: input.src,
                    body: Body {
                        id: self.id,
                        in_reply_to: input.body.id,
                        msg_type: MessageType::EchoOk { echo },
                    },
                };
                serde_json::to_writer(&mut *output, &reply)
                    .context("serialize response to init")?;
                output.write_all(b"\n").context("write trailing newline")?;
                self.id += 1;
            }
            MessageType::InitOk { .. } => bail!("received init_ok message"),
            MessageType::EchoOk { .. } => {}
        }

        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    main_loop(EchoNode { id: 0 })
}
