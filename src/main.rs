use futures_util::{StreamExt, SinkExt};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use warp::Filter;

#[tokio::main]
async fn main() {

    let (tx, _rx) = broadcast::channel(100);
    let tx = Arc::new(Mutex::new(tx));

    let tx_ws = tx.clone();
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let tx = tx_ws.clone();
            ws.on_upgrade(move |websocket| handle_connection(websocket, tx))
        });

        let static_route = warp::path::end()
        .and(warp::fs::file("index.html"));

        let routes = ws_route.or(static_route);
    println!("El servidor se escucha en -> 127.0.0.1:8080");

    warp::serve(routes)
        .run(([127, 0, 0, 1], 8080))
        .await;
}
async fn handle_connection(ws: warp::ws::WebSocket, tx: Arc<Mutex<broadcast::Sender<String>>>) {
    let (mut ws_sender, mut ws_receiver) = ws.split();

    let mut rx = tx.lock().unwrap().subscribe();

    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if ws_sender.send(warp::ws::Message::text(msg)).await.is_err() {
                break;
            }
        }
    });

    while let Some(result) = ws_receiver.next().await {
        match result {
            Ok(message) => {
                if let Ok(text) = message.to_str() {
                    println!("Mensaje recibido: {}", text);
                    tx.lock().unwrap().send(text.to_string()).expect("Falló la transmisión");
                }
            },
            Err(e) => {
                eprintln!("Error recibiendo el mensaje: {}", e);
                break;
            }
        }
    }
}