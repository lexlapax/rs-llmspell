use crate::config::WebConfig;
use llmspell_kernel::api::KernelHandle;
use metrics_exporter_prometheus::PrometheusHandle;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppState {
    pub kernel: Arc<Mutex<KernelHandle>>,
    pub metrics_recorder: PrometheusHandle,
    pub config: WebConfig,
    pub runtime_config: Arc<tokio::sync::RwLock<llmspell_config::env::EnvRegistry>>,
    pub config_store: Option<Arc<llmspell_storage::backends::sqlite::SqliteKVStorage>>,
    pub static_config_path: Option<std::path::PathBuf>,
}
