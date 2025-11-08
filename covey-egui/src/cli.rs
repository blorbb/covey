use std::{
    io::{self, BufRead as _, Read as _, Write as _},
    sync::{
        Arc,
        mpsc::{self, RecvError, TryRecvError},
    },
    thread,
};

use interprocess::local_socket::{
    GenericNamespaced, ListenerOptions, Stream, ToNsName as _,
    traits::{ListenerExt as _, Stream as _},
};
use parking_lot::Mutex;

// may have strings inside later, so not Copy
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Message {
    Open,
    Exit,
}

#[derive(Debug, Clone)]
pub struct Receiver {
    rx: Arc<mpsc::Receiver<Message>>,
    last_msg: Arc<Mutex<Option<Message>>>,
}

impl Receiver {
    pub fn recv(&self) -> Message {
        match self.rx.recv() {
            Ok(msg) => {
                *self.last_msg.lock() = Some(msg.clone());
                msg
            }
            Err(RecvError) => panic!("cli listener should not be disconnected"),
        }
    }

    pub fn try_recv(&self) -> Option<Message> {
        match self.rx.try_recv() {
            Ok(msg) => {
                *self.last_msg.lock() = Some(msg.clone());
                Some(msg)
            }
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => panic!("cli listener should not be disconnected"),
        }
    }

    /// Gets the last message that was handled.
    ///
    /// There may be other messages in the receiving channel but haven't been handled
    /// with [`Self::try_recv`] yet.
    pub fn last_handled_msg(&self) -> Option<Message> {
        self.last_msg.lock().clone()
    }
}

/// Makes a listener for CLI messages, returning `Ok(None)` if covey is already
/// open and this process should stop, and `Ok(Some(rx))` if this is the primary instance.
pub fn listener() -> io::Result<Option<Receiver>> {
    let name = "covey.sock".to_ns_name::<GenericNamespaced>()?;

    let listener = match ListenerOptions::new().name(name.clone()).create_sync() {
        Err(e) if e.kind() == io::ErrorKind::AddrInUse => {
            tracing::info!("address in use");
            // Connect to the existing socket and ask it to open
            let mut conn = Stream::connect(name)?;
            conn.write_all(b"open\n")?;

            // Wait for a response just to confirm
            conn.read_to_end(&mut Vec::new())?;

            return Ok(None);
        }
        x => x?,
    };

    let (tx, rx) = mpsc::channel::<Message>();
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

    return Ok(Some(Receiver {
        rx: Arc::new(rx),
        last_msg: Arc::new(Mutex::new(None)),
    }));

    fn handle_msg(msg: Stream, tx: &mpsc::Sender<Message>) -> anyhow::Result<()> {
        let mut request = String::new();
        let mut msg = io::BufReader::new(msg); // Needed for read_line

        msg.read_line(&mut request)?;
        request.truncate(request.trim_end().len()); // Remove trailing newline
        tracing::info!("received request {request:?}");

        match &*request.trim() {
            "open" => {
                tx.send(Message::Open)?;
            }
            "exit" => {
                tx.send(Message::Exit)?;
            }
            _ => {
                anyhow::bail!("unknown message {request:?}");
            }
        }

        Ok(())
    }
}
