use std::future::Future;

use crate::{
    list::ListItemCallbacks, plugin_lock::PluginLock, proto, sql, store, Action, Details, Hotkey,
    Input, List, Result,
};

pub trait Plugin: Sized + Send + Sync + 'static {
    fn new(toml: String) -> impl Future<Output = Result<Self>> + Send;

    fn query(&self, query: String) -> impl Future<Output = Result<List>> + Send;

    fn details() -> Details {
        Details::new()
    }
}

type TonicResult<T> = Result<tonic::Response<T>, tonic::Status>;

#[tonic::async_trait]
impl<T> proto::plugin_server::Plugin for PluginLock<T>
where
    T: Plugin,
{
    async fn initialise(
        &self,
        request: tonic::Request<proto::InitialiseRequest>,
    ) -> TonicResult<()> {
        let request = request.into_inner();
        let mut guard = self.write().await;
        sql::init(&request.sqlite_url)
            .await
            .map_err(into_tonic_status)?;
        let plugin = T::new(request.toml).await.map_err(into_tonic_status)?;
        *guard = Some(plugin);

        Ok(tonic::Response::new(()))
    }

    async fn query(
        &self,
        request: tonic::Request<proto::QueryRequest>,
    ) -> TonicResult<proto::QueryResponse> {
        let list = self
            .force_read()
            .await
            .query(request.into_inner().query)
            .await
            .map_err(into_tonic_status)?;

        Ok(tonic::Response::new(store::store_query_result(list)))
    }

    async fn activate(
        &self,
        request: tonic::Request<proto::ActivationRequest>,
    ) -> TonicResult<proto::ActivationResponse> {
        activate_using(request.into_inner(), ListItemCallbacks::activate).await
    }

    async fn alt_activate(
        &self,
        request: tonic::Request<proto::ActivationRequest>,
    ) -> TonicResult<proto::ActivationResponse> {
        activate_using(request.into_inner(), ListItemCallbacks::alt_activate).await
    }

    async fn hotkey_activate(
        &self,
        request: tonic::Request<proto::HotkeyActivationRequest>,
    ) -> TonicResult<proto::ActivationResponse> {
        let request = request.into_inner();
        let hotkey = request.hotkey;
        let cx = request.request;
        activate_using(cx, |callback| {
            callback.hotkey_activate(Hotkey::from_proto(hotkey))
        })
        .await
    }

    async fn complete(
        &self,
        request: tonic::Request<proto::ActivationRequest>,
    ) -> TonicResult<proto::CompletionResponse> {
        let id = request.into_inner().selection_id;
        let callback = store::fetch_callbacks_of(id).ok_or(tonic::Status::data_loss(format!(
            "failed to fetch callback of list item with id {id}"
        )))?;

        let new_input = callback
            .complete()
            .await
            .map(|input| proto::CompletionResponse {
                input: input.map(Input::into_proto),
            });

        map_result(new_input)
    }

    async fn details(&self, _: tonic::Request<()>) -> TonicResult<proto::DetailsResponse> {
        Ok(tonic::Response::new(T::details().into_proto()))
    }
}

async fn activate_using<Fut>(
    request: proto::ActivationRequest,
    function: impl FnOnce(ListItemCallbacks) -> Fut,
) -> TonicResult<proto::ActivationResponse>
where
    Fut: Future<Output = Result<Vec<Action>>>,
{
    let id = request.selection_id;
    let callback = store::fetch_callbacks_of(id).ok_or(tonic::Status::data_loss(format!(
        "failed to fetch callback of list item with id {id}"
    )))?;
    let response = function(callback).await.map(|a| proto::ActivationResponse {
        actions: a.into_iter().map(Action::into_proto).collect(),
    });

    map_result(response)
}

fn map_result<T>(result: Result<T>) -> TonicResult<T> {
    match result {
        Ok(response) => Ok(tonic::Response::new(response)),
        Err(err) => Err(into_tonic_status(err)),
    }
}

fn into_tonic_status(e: anyhow::Error) -> tonic::Status {
    tonic::Status::unknown(
        e.chain()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n"),
    )
}
