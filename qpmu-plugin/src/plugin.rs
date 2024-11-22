use std::future::Future;

use crate::{
    plugin_lock::PluginLock, proto, sql, Action, ActivationContext, Hotkey, Input, List, Result,
};

pub trait Plugin: Sized + Send + Sync + 'static {
    fn new(toml: String) -> impl Future<Output = Result<Self>> + Send;

    fn query(&self, query: String) -> impl Future<Output = Result<List>> + Send;

    fn activate(&self, cx: ActivationContext) -> impl Future<Output = Result<Vec<Action>>> + Send;

    /// What to do on an alternate activation (alt+enter by default).
    ///
    /// Defaults to doing nothing.
    fn alt_activate(
        &self,
        _cx: ActivationContext,
    ) -> impl Future<Output = Result<Vec<Action>>> + Send {
        async { Ok(vec![]) }
    }

    fn hotkey_activate(
        &self,
        _hotkey: Hotkey,
        _cx: ActivationContext,
    ) -> impl Future<Output = Result<Vec<Action>>> + Send {
        async { Ok(vec![]) }
    }

    fn complete(
        &self,
        _cx: ActivationContext,
    ) -> impl Future<Output = Result<Option<Input>>> + Send {
        async { Ok(None) }
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
        let response = self
            .force_read()
            .await
            .query(request.into_inner().query)
            .await;

        map_result(response.map(List::into_proto))
    }

    async fn activate(
        &self,
        request: tonic::Request<proto::ActivationRequest>,
    ) -> TonicResult<proto::ActivationResponse> {
        let plugin = self.force_read().await;
        activate_using(request.into_inner(), |req| T::activate(&plugin, req)).await
    }

    async fn alt_activate(
        &self,
        request: tonic::Request<proto::ActivationRequest>,
    ) -> TonicResult<proto::ActivationResponse> {
        let plugin = self.force_read().await;
        activate_using(request.into_inner(), |req| T::alt_activate(&plugin, req)).await
    }

    async fn hotkey_activate(
        &self,
        request: tonic::Request<proto::HotkeyActivationRequest>,
    ) -> TonicResult<proto::ActivationResponse> {
        let plugin = self.force_read().await;
        let request = request.into_inner();
        let hotkey = request.hotkey;
        let cx = request.request;
        activate_using(cx, |req| {
            T::hotkey_activate(&plugin, Hotkey::from_proto(hotkey), req)
        })
        .await
    }

    async fn complete(
        &self,
        request: tonic::Request<proto::ActivationRequest>,
    ) -> TonicResult<proto::CompletionResponse> {
        let request = ActivationContext::from_request(request.into_inner());

        map_result(
            T::complete(&*self.force_read().await, request)
                .await
                .map(|input| proto::CompletionResponse {
                    input: input.map(Input::into_proto),
                }),
        )
    }
}

async fn activate_using<Fut>(
    request: proto::ActivationRequest,
    function: impl FnOnce(ActivationContext) -> Fut,
) -> TonicResult<proto::ActivationResponse>
where
    Fut: Future<Output = Result<Vec<Action>>>,
{
    let request = ActivationContext::from_request(request);
    sql::increment_frequency_table(&request.item.title)
        .await
        .map_err(|e| tonic::Status::unknown(e.to_string()))?;

    map_result(
        function(request)
            .await
            .map(|actions| proto::ActivationResponse {
                actions: actions.into_iter().map(Action::into_proto).collect(),
            }),
    )
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
