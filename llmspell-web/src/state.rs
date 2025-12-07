use std::sync::Arc;
use tokio::sync::Mutex;
use llmspell_kernel::api::KernelHandle;
use metrics_exporter_prometheus::PrometheusHandle;
use crate::config::WebConfig;

#[derive(Clone)]
pub struct AppState {
    pub kernel: Arc<Mutex<KernelHandle>>,
    pub metrics_recorder: PrometheusHandle,
    pub config: WebConfig,
    pub runtime_config: Arc<tokio::sync::RwLock<llmspell_config::env::EnvRegistry>>,
}
