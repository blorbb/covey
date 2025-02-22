use std::process;

use parking_lot::Mutex;
use tokio::{net::TcpListener, sync::RwLock};
use tonic::transport::Server;

use crate::{Plugin, proto::plugin_server::PluginServer, store::ListItemStore};

pub(crate) struct ServerState<P> {
    pub(crate) plugin: RwLock<Option<P>>,
    pub(crate) list_item_store: Mutex<ListItemStore>,
}

impl<T: Plugin> ServerState<T> {
    pub(crate) fn new_empty() -> Self {
        Self {
            plugin: RwLock::new(None),
            list_item_store: Mutex::new(ListItemStore::new()),
        }
    }
}

/// Starts up the server with a specified plugin implementation.
///
/// The plugin id should be `env!("CARGO_PKG_NAME")`.
///
/// This will start a single-threaded tokio runtime.
pub fn run_server<T: Plugin>(plugin_id: &'static str) -> ! {
    crate::PLUGIN_ID
        .set(plugin_id)
        .expect("plugin id should only be set from main");
    let result = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| anyhow::anyhow!(e))
        .and_then(|rt| {
            rt.block_on(async {
                // if port 0 is provided, asks the OS for a port
                // https://github.com/hyperium/tonic/blob/master/tests/integration_tests/tests/timeout.rs#L77-L89
                let listener = TcpListener::bind("[::1]:0").await?;
                let port = listener.local_addr()?.port();

                // print port for covey to read
                println!("{port}");

                Server::builder()
                    .add_service(PluginServer::new(ServerState::<T>::new_empty()))
                    .serve_with_incoming(tokio_stream::wrappers::TcpListenerStream::new(listener))
                    .await?;

                Ok(())
            })
        });

    match result {
        Ok(()) => process::exit(0),
        Err(e) => {
            print_error(&e);
            process::exit(1)
        }
    }
}

fn print_error(e: &anyhow::Error) {
    let err_string = e
        .chain()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    eprintln!("{err_string}");
}
