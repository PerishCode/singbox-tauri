use utoipa::OpenApi;

use crate::app::handlers::control::{
    AppEventAccepted, AppEventRequest, ControlSnapshotResponse, ControlStateResponse,
    HealthResponse, LocalNetworkResponse,
};
use crate::app::services::network::{
    LocalNetworkSnapshot, NetworkConflict, NetworkConflictLevel, NetworkDefaultRoute,
    NetworkDiagnostics, NetworkDnsResolver, NetworkInterfaceSummary, NetworkPortBinding,
    NetworkProcessSignal, NetworkProxyStatus, NetworkReadiness,
};
use crate::app::services::singbox::{SingboxBootstrapReport, SingboxCheck, SingboxRuntimeStatus};
use crate::app::services::subscription::{
    SubscriptionDecryptState, SubscriptionFetchState, SubscriptionKeyState, SubscriptionSnapshot,
};
use crate::app::state::{AppLifecycle, AppRunMode};
use crate::runtime_paths::{RuntimeMode, RuntimePaths};
use crate::singbox::process::ProcessStatus;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::app::handlers::control::health,
        crate::app::handlers::control::state,
        crate::app::handlers::control::snapshot,
        crate::app::handlers::control::network,
        crate::app::handlers::control::start,
        crate::app::handlers::control::stop,
        crate::app::handlers::control::restart,
        crate::app::handlers::control::app_log,
        crate::app::handlers::control::singbox_log,
        crate::app::handlers::control::append_event,
    ),
    components(schemas(
        HealthResponse,
        ControlStateResponse,
        ControlSnapshotResponse,
        LocalNetworkResponse,
        RuntimePaths,
        RuntimeMode,
        LocalNetworkSnapshot,
        NetworkReadiness,
        NetworkConflictLevel,
        NetworkConflict,
        NetworkDefaultRoute,
        NetworkDiagnostics,
        NetworkDnsResolver,
        NetworkInterfaceSummary,
        NetworkProcessSignal,
        NetworkPortBinding,
        NetworkProxyStatus,
        SubscriptionKeyState,
        SubscriptionFetchState,
        SubscriptionDecryptState,
        SubscriptionSnapshot,
        SingboxCheck,
        SingboxBootstrapReport,
        SingboxRuntimeStatus,
        ProcessStatus,
        AppLifecycle,
        AppRunMode,
        AppEventRequest,
        AppEventAccepted,
    ))
)]
pub struct ApiDoc;
