use crate::models::RealtimeEvent;
use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
    routing::get,
    Router,
};
use futures::stream::{self, Stream};
use std::{convert::Infallible, sync::Arc};
use tokio::sync::broadcast;

pub type EventBroadcaster = broadcast::Sender<RealtimeEvent>;

pub struct RealtimeState {
    pub broadcaster: EventBroadcaster,
}

impl RealtimeState {
    pub fn new() -> Self {
        let (broadcaster, _) = broadcast::channel(1000);
        Self { broadcaster }
    }
    
    pub fn broadcast(&self, event: RealtimeEvent) {
        let _ = self.broadcaster.send(event);
    }
}

pub fn create_sse_router(state: Arc<RealtimeState>) -> Router {
    Router::new()
        .route("/api/events", get(sse_handler))
        .with_state(state)
}

async fn sse_handler(
    State(state): State<Arc<RealtimeState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let mut rx = state.broadcaster.subscribe();
    
    let stream = stream::unfold(rx, |mut rx| async move {
        match rx.recv().await {
            Ok(event) => {
                let json = serde_json::to_string(&event).ok()?;
                let sse_event = Event::default().data(json);
                Some((Ok(sse_event), rx))
            }
            Err(_) => None,
        }
    });
    
    Sse::new(stream).keep_alive(KeepAlive::default())
}

// Helper functions to broadcast events from other parts of the application
pub fn broadcast_email_received(broadcaster: &EventBroadcaster, email: crate::models::Email) {
    let event = RealtimeEvent::EmailReceived { email };
    let _ = broadcaster.send(event);
}

pub fn broadcast_email_updated(broadcaster: &EventBroadcaster, email: crate::models::Email) {
    let event = RealtimeEvent::EmailUpdated { email };
    let _ = broadcaster.send(event);
}

pub fn broadcast_email_state_changed(
    broadcaster: &EventBroadcaster,
    email_id: uuid::Uuid,
    from: crate::models::EmailState,
    to: crate::models::EmailState,
) {
    let event = RealtimeEvent::EmailStateChanged { email_id, from, to };
    let _ = broadcaster.send(event);
}
