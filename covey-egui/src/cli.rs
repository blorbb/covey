use std::{
    io::{self, BufRead as _, Read as _, Write as _},
    thread,
};

use clap::{Parser, Subcommand};
use interprocess::local_socket::{
    GenericNamespaced, ListenerOptions, Stream, ToNsName as _,
    traits::{ListenerExt as _, Stream as _},
};
use tokio::sync::mpsc::{self, error::TryRecvError};

use crate::GuiSettings;

// may have strings inside later, so not Copy
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Action {
    Open,
    OpenAndStay,
    Exit,
}

#[derive(Debug)]
pub struct Receiver {
    rx: mpsc::UnboundedReceiver<Action>,
    last_msg: Option<Action>,
}

impl Receiver {
    /// Cancel safe.
    #[must_use]
    pub async fn recv(&mut self) -> Action {
        let msg = self
            .rx
            .recv()
            .await
            .expect("cli listener should not be disconnected");
        self.last_msg = Some(msg.clone());
        msg
    }

    #[must_use]
    pub fn try_recv(&mut self) -> Option<Action> {
        match self.rx.try_recv() {
            Ok(msg) => {
                self.last_msg = Some(msg.clone());
                Some(msg)
            }
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => panic!("cli listener should not be disconnected"),
        }
    }

    /// Gets the last message that was handled.
    ///
    /// There may be other messages in the receiving channel but haven't been
    /// handled with [`Self::try_recv`] yet.
    #[must_use]
    pub fn last_handled_msg(&self) -> Option<&Action> {
        self.last_msg.as_ref()
    }
}

/// Makes a listener for CLI messages, returning `Ok(None)` this process should
/// stop, and `Ok(Some(rx))` if this is the primary instance.
///
/// Also parses CLI arguments, exiting with a help message if it fails.
pub fn listener() -> io::Result<Option<(GuiSettings, Receiver)>> {
    let args = Args::parse();
    let cmd = args.cmd.unwrap_or_default();

    let name = "covey.sock".to_ns_name::<GenericNamespaced>()?;

    let listener = match ListenerOptions::new().name(name.clone()).create_sync() {
        // Another instance already open, send message to that instance
        Err(e) if e.kind() == io::ErrorKind::AddrInUse => {
            tracing::info!("address in use");
            let mut conn = Stream::connect(name)?;

            let msg = match cmd {
                CliCommands::Open { stay_open: false } => {
                    tracing::info!("opening existing instance");
                    b"open\n".as_slice()
                }
                CliCommands::Open { stay_open: true } => {
                    tracing::info!("opening existing instance and keeping open");
                    b"open stay\n"
                }
                CliCommands::Exit => {
                    tracing::info!("closing existing instance");
                    b"exit\n"
                }
            };
            conn.write_all(msg)?;

            // Wait for a response just to confirm
            conn.read_to_end(&mut Vec::new())?;
            tracing::info!("confirmation received");

            return Ok(None);
        }
        x => x?,
    };

    // This is the primary instance, initialise.
    match cmd {
        CliCommands::Open { stay_open } => {
            let (tx, rx) = mpsc::unbounded_channel::<Action>();
            thread::spawn(move || {
                for msg in listener.incoming() {
                    match msg {
                        Ok(msg) => {
                            if let Err(e) = handle_msg(msg, &tx) {
                                tracing::error!("error handling message from cli: {e}")
                            }
                        }
                        Err(e) => {
                            tracing::error!("error receiving message from cli: {e}")
                        }
                    }
                }
            });

            return Ok(Some((
                GuiSettings {
                    close_on_blur: !stay_open,
                },
                Receiver { rx, last_msg: None },
            )));
        }
        CliCommands::Exit => {
            tracing::error!("no existing instance to exit from");
            return Ok(None);
        }
    }

    fn handle_msg(msg: Stream, tx: &mpsc::UnboundedSender<Action>) -> anyhow::Result<()> {
        let mut request = String::new();
        let mut msg = io::BufReader::new(msg); // Needed for read_line

        msg.read_line(&mut request)?;
        request.truncate(request.trim_end().len()); // Remove trailing newline
        tracing::info!("received request {request:?}");

        match &*request.trim() {
            "open" => {
                tx.send(Action::Open)?;
            }
            "open stay" => {
                tx.send(Action::OpenAndStay)?;
            }
            "exit" => {
                tx.send(Action::Exit)?;
            }
            _ => {
                anyhow::bail!("unknown message {request:?}");
            }
        }

        Ok(())
    }
}

#[derive(Parser)]
struct Args {
    #[command(subcommand)]
    cmd: Option<CliCommands>,
}

#[derive(Subcommand, Debug, PartialEq, Eq)]
enum CliCommands {
    Open {
        #[arg(short, long)]
        stay_open: bool,
    },
    Exit,
}

impl Default for CliCommands {
    fn default() -> Self {
        Self::Open { stay_open: false }
    }
}
