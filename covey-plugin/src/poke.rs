use tokio::sync::watch;

#[derive(Debug, Clone)]
pub struct Sender(watch::Sender<()>);

impl Sender {
    /// Pokes all receivers.
    ///
    /// Does nothing if all receivers are dropped.
    pub fn poke(&self) {
        _ = self.0.send(());
    }
}

#[derive(Debug, Clone)]
pub struct Receiver(watch::Receiver<()>);

impl Receiver {
    /// Waits for a poke, then marks it as seen.
    ///
    /// Returns an error if and only if all senders have been dropped.
    pub async fn poked(&mut self) -> Result<(), watch::error::RecvError> {
        self.0.changed().await
    }
}

pub fn channel() -> (Sender, Receiver) {
    let (tx, rx) = watch::channel(());
    (Sender(tx), Receiver(rx))
}
