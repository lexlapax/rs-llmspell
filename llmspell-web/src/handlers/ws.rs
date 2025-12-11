use crate::state::AppState;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use tracing::info;

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    info!("[WS] WebSocket connection established");

    // Get event bus from kernel -> session manager
    let event_bus = {
        let kernel = state.kernel.lock().await;
        let bus = kernel.session_manager().event_bus();
        info!(
            "[WS] Got EventBus from SessionManager, broadcast_tx receiver_count before subscribe: {}",
            bus.receiver_count()
        );
        (**bus).clone()
    };

    // Subscribe to all events
    let mut rx = event_bus.subscribe_all();
    info!(
        "[WS] Subscribed to EventBus, receiver_count after subscribe: {}",
        event_bus.receiver_count()
    );

    // Spawn a task to forward events to the websocket
    let (mut sender, mut receiver) = socket.split();

    let mut send_task = tokio::spawn(async move {
        info!("[WS] Send task started, waiting for events...");
        while let Ok(event) = rx.recv().await {
            info!("[WS] Received event from EventBus: {}", event.event_type);
            // Serialize event to JSON
            if let Ok(json) = serde_json::to_string(&event) {
                if sender.send(Message::Text(json)).await.is_err() {
                    info!("[WS] Failed to send event to WebSocket, closing");
                    break;
                }
            }
        }
        info!("[WS] Send task ended");
    });

    // Wait for the client to disconnect
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(_msg)) = receiver.next().await {
            // We can handle client messages here if needed (e.g. ping/pong, subscription filters)
            // For now, just keep the connection alive
        }
    });

    // If any one of the tasks exit, abort the other
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };

    info!("WebSocket connection closed");
}
