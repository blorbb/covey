use std::process;

use tokio::net::TcpListener;
use tonic::transport::Server;

use crate::{plugin_lock::PluginLock, proto::plugin_server::PluginServer, Plugin};

pub fn run_server<T: Plugin>() -> ! {
    let result = tokio::runtime::Runtime::new()
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
                    .add_service(PluginServer::new(PluginLock::<T>::new_empty()))
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
