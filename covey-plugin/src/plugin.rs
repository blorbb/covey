use std::future::Future;

use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use crate::{List, Menu, Result, manifest::ManifestDeserialization, proto, server::ServerState};

pub trait Plugin: Sized + Send + Sync + 'static {
    /// The user's configuration for this plugin.
    ///
    /// Use `()` if this plugin has no configuration.
    type Config: ManifestDeserialization;

    fn new(config: Self::Config) -> impl Future<Output = Result<Self>> + Send;

    fn query(&self, query: String) -> impl Future<Output = Result<List>> + Send;
}

type TonicResult<T> = Result<tonic::Response<T>, tonic::Status>;

#[tonic::async_trait]
impl<T> proto::plugin_server::Plugin for ServerState<T>
where
    T: Plugin,
{
    async fn initialise(
        &self,
        request: tonic::Request<proto::InitialiseRequest>,
    ) -> TonicResult<()> {
        let request = request.into_inner();

        let mut guard = self.plugin.write().await;

        let config = ManifestDeserialization::try_from_input(&request.json)
            .map_err(|e| tonic::Status::invalid_argument(e.to_string()))?;
        let plugin = T::new(config).await.map_err(into_tonic_status)?;

        *guard = Some(plugin);

        Ok(tonic::Response::new(()))
    }

    async fn query(
        &self,
        request: tonic::Request<proto::QueryRequest>,
    ) -> TonicResult<proto::QueryResponse> {
        let list = self
            .plugin
            .read()
            .await
            .as_ref()
            .expect("plugin has not been initialised")
            .query(request.into_inner().query)
            .await
            .map_err(into_tonic_status)?;

        Ok(tonic::Response::new(
            self.list_item_store.lock().store_query_result(list),
        ))
    }

    type ActivateStream = ReceiverStream<Result<proto::ActivationResponse, tonic::Status>>;

    async fn activate(
        &self,
        request: tonic::Request<proto::ActivationRequest>,
    ) -> TonicResult<Self::ActivateStream> {
        let request = request.into_inner();
        let id = request.selection_id;
        let callbacks =
            self.list_item_store
                .lock()
                .fetch_callbacks_of(id)
                .ok_or(tonic::Status::data_loss(format!(
                    "failed to fetch callback of list item with id {id}"
                )))?;

        let (tx, rx) = mpsc::channel(4);
        let menu = Menu { sender: tx };

        // tonic plugin requires methods to be Send + Sync, but this
        // is annoying. spawn_local makes this future no longer require
        // Send + Sync.
        tokio::task::spawn_local(async move {
            callbacks.call_command(&request.command_name, menu).await;
        })
        .await
        // JoinHandle resolves to an err if the task panicked.
        .map_err(|e| tonic::Status::internal(e.to_string()))?;

        Ok(tonic::Response::new(ReceiverStream::new(rx)))
    }
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "easier to only use path when mapping"
)]
fn into_tonic_status(e: anyhow::Error) -> tonic::Status {
    tonic::Status::unknown(
        e.chain()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n"),
    )
}
