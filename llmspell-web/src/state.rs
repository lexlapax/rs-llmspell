use std::sync::Arc;
use tokio::sync::Mutex;
use llmspell_kernel::api::KernelHandle;

#[derive(Clone)]
pub struct AppState {
    pub kernel: Arc<Mutex<KernelHandle>>,
}
