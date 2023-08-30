use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::Method,
    response::Response,
    routing::get,
    Router, Server,
};
use serde_json::json;
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};
use sysinfo::{CpuExt, DiskExt, System, SystemExt};
use tokio::time::{interval, Duration};
use tower_http::cors::CorsLayer;

#[derive(Clone)]
struct AppState {
    sys: Arc<Mutex<System>>,
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    let mut ticker = interval(Duration::from_secs(1));
    loop {
        ticker.tick().await;

        let cpu_data = get_cpu_data(&state).await;
        let ram_data = get_ram_data(&state).await;
        let swap_data = get_swap_data(&state).await;
        let disk_data = get_disk_data(&state).await;

        let json_str = serde_json::to_string(&json!({
            "cpu": cpu_data,
            "ram": ram_data,
            "swap": swap_data,
            "disk": disk_data
        }))
        .expect("Failed to serialize data");

        if let Err(_) = socket.send(Message::Text(json_str)).await {
            eprintln!("Error sending data to client");
            break;
        }
    }
}

async fn get_disk_data(state: &AppState) -> Option<BTreeMap<String, f32>> {
    let mut sys = state.sys.lock().unwrap();
    sys.refresh_disks();

    sys.disks().get(1).map(|d| {
        let mut map = BTreeMap::new();
        map.insert("total".to_string(), d.total_space() as f32);
        map.insert("available".to_string(), d.available_space() as f32);
        map.insert(
            "used".to_string(),
            (d.total_space() - d.available_space()) as f32,
        );
        map
    })
}

async fn get_cpu_data(state: &AppState) -> BTreeMap<usize, f32> {
    let mut sys = state.sys.lock().unwrap();
    sys.refresh_cpu();

    sys.cpus()
        .iter()
        .enumerate()
        .map(|(i, c)| (i, c.cpu_usage()))
        .collect()
}

async fn get_ram_data(state: &AppState) -> BTreeMap<String, f32> {
    let mut sys = state.sys.lock().unwrap();
    sys.refresh_memory();

    let total = sys.total_memory();
    let used = sys.used_memory();
    let free = sys.free_memory();

    let mut map = BTreeMap::new();
    map.insert("total".to_string(), total as f32);
    map.insert("used".to_string(), used as f32);
    map.insert("free".to_string(), free as f32);
    map
}

async fn get_swap_data(state: &AppState) -> BTreeMap<String, f32> {
    let mut sys = state.sys.lock().unwrap();
    sys.refresh_memory();

    let total = sys.total_swap();
    let used = sys.used_swap();
    let free = sys.free_swap();

    let mut map = BTreeMap::new();
    map.insert("total".to_string(), total as f32);
    map.insert("used".to_string(), used as f32);
    map.insert("free".to_string(), free as f32);
    map
}

async fn handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    let cloned_state = state.clone();
    ws.on_upgrade(move |socket| handle_socket(socket, cloned_state))
}

#[tokio::main]
async fn main() {
    let state = AppState {
        sys: Arc::new(Mutex::new(System::new_all())),
    };
    let router = Router::new()
        .route("/ws", get(handler))
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET, Method::OPTIONS, Method::POST])
                .allow_origin(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any),
        );
    let server = Server::bind(&"0.0.0.0:9000".parse().unwrap()).serve(router.into_make_service());
    let addr = server.local_addr();
    println!("Listening on http://{}", addr);

    server.await.expect("Failed to start server");
}
