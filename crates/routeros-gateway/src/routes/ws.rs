use std::collections::HashMap;
use std::sync::Arc;
use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Query, State};
use axum::response::IntoResponse;
use futures::{SinkExt, StreamExt};
use routeros_core::build_command;
use serde::Deserialize;
use serde_json::json;
use tokio::sync::mpsc;

use crate::state::{AppState, RouterTarget};

#[derive(Deserialize, Debug)]
pub struct WsQuery {
    pub token: Option<String>,
}

#[derive(Deserialize, Debug)]
struct WsIncoming {
    pub action: String,
    pub token: Option<String>,
    pub tag: Option<String>,
    pub router: Option<RouterTarget>,
    pub router_id: Option<String>,
    pub command: Option<String>,
    #[serde(default)]
    pub params: HashMap<String, String>,
    pub interface: Option<String>,
}

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Query(query): Query<WsQuery>,
    State(st): State<Arc<AppState>>,
) -> impl IntoResponse {
    let pre_auth = query.token.as_ref().is_some_and(|t| t == &st.token);
    ws.on_upgrade(move |socket| handle_ws(socket, st, pre_auth))
}

async fn handle_ws(socket: WebSocket, st: Arc<AppState>, mut authenticated: bool) {
    let (mut ws_sink, mut ws_stream) = socket.split();

    // Channel for outbound messages to ws_sink
    let (out_tx, mut out_rx) = mpsc::unbounded_channel::<String>();

    // Background sender task
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = out_rx.recv().await {
            if ws_sink.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Active streams: key = tag, value = cancellation sender / join handle abort
    let mut active_streams: HashMap<String, tokio::task::JoinHandle<()>> = HashMap::new();

    // Send connection greeting
    let _ = out_tx.send(json!({
        "event": "connected",
        "service": "mikrotik-universal-rust-gateway",
        "authenticated": authenticated
    }).to_string());

    while let Some(Ok(msg)) = ws_stream.next().await {
        let text = match msg {
            Message::Text(t) => t,
            Message::Close(_) => break,
            Message::Ping(p) => {
                let _ = out_tx.send(json!({ "event": "pong", "payload": p }).to_string());
                continue;
            }
            _ => continue,
        };

        let req: WsIncoming = match serde_json::from_str(&text) {
            Ok(parsed) => parsed,
            Err(e) => {
                let _ = out_tx.send(json!({ "event": "error", "error": format!("Invalid JSON: {e}") }).to_string());
                continue;
            }
        };

        let tag = req.tag.clone().unwrap_or_else(|| "default".into());

        // Action: auth
        if req.action == "auth" {
            if let Some(t) = req.token {
                if t == st.token {
                    authenticated = true;
                    let _ = out_tx.send(json!({ "event": "auth_ok", "tag": tag }).to_string());
                } else {
                    let _ = out_tx.send(json!({ "event": "auth_error", "tag": tag, "error": "Invalid token" }).to_string());
                }
            }
            continue;
        }

        // Action: ping
        if req.action == "ping" {
            let _ = out_tx.send(json!({ "event": "pong", "tag": tag, "timestamp": chrono_now_ms() }).to_string());
            continue;
        }

        if !authenticated {
            let _ = out_tx.send(json!({ "event": "unauthorized", "tag": tag, "error": "Please authenticate via query ?token= or action 'auth'" }).to_string());
            continue;
        }

        match req.action.as_str() {
            "command" => {
                let cmd_str = match req.command {
                    Some(c) if !c.trim().is_empty() => c,
                    _ => {
                        let _ = out_tx.send(json!({ "event": "error", "tag": tag, "error": "Missing 'command' field" }).to_string());
                        continue;
                    }
                };

                let st_clone = st.clone();
                let out_tx_clone = out_tx.clone();
                let tag_clone = tag.clone();

                tokio::spawn(async move {
                    let client_res = st_clone.resolve_client(req.router.as_ref(), req.router_id.as_deref()).await;
                    let client = match client_res {
                        Ok(c) => c,
                        Err(e) => {
                            let _ = out_tx_clone.send(json!({ "event": "command_error", "tag": tag_clone, "error": e.to_string() }).to_string());
                            return;
                        }
                    };

                    let words = build_command(&cmd_str, req.params.into_iter());
                    match client.run(words).await {
                        Ok(rows) => {
                            let data: Vec<_> = rows.into_iter().map(|r| r.attrs).collect();
                            let _ = out_tx_clone.send(json!({
                                "event": "command_result",
                                "tag": tag_clone,
                                "success": true,
                                "count": data.len(),
                                "data": data
                            }).to_string());
                        }
                        Err(e) => {
                            let _ = out_tx_clone.send(json!({
                                "event": "command_error",
                                "tag": tag_clone,
                                "error": e.to_string()
                            }).to_string());
                        }
                    }
                });
            }

            "subscribe_traffic" => {
                let iface = req.interface.unwrap_or_else(|| "ether1".into());
                let st_clone = st.clone();
                let out_tx_clone = out_tx.clone();
                let stream_tag = tag.clone();
                let iface_clone = iface.clone();

                // Abort any existing stream with this tag
                if let Some(old) = active_streams.remove(&stream_tag) {
                    old.abort();
                }

                let handle = tokio::spawn(async move {
                    let client_res = st_clone.resolve_client(req.router.as_ref(), req.router_id.as_deref()).await;
                    let client = match client_res {
                        Ok(c) => c,
                        Err(e) => {
                            let _ = out_tx_clone.send(json!({ "event": "stream_error", "tag": stream_tag, "error": e.to_string() }).to_string());
                            return;
                        }
                    };

                    let words = build_command("/interface/monitor-traffic", [
                        ("interface", iface_clone.as_str()),
                        ("once", ""),
                    ]);

                    // Continuous polling interval for live interface traffic over WebSocket
                    loop {
                        match client.run(words.clone()).await {
                            Ok(rows) => {
                                if let Some(first) = rows.into_iter().next() {
                                    let sent = out_tx_clone.send(json!({
                                        "event": "traffic_frame",
                                        "tag": stream_tag,
                                        "interface": iface_clone,
                                        "data": first.attrs
                                    }).to_string());
                                    if sent.is_err() {
                                        break;
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = out_tx_clone.send(json!({
                                    "event": "stream_error",
                                    "tag": stream_tag,
                                    "error": e.to_string()
                                }).to_string());
                                break;
                            }
                        }
                        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
                    }
                });

                active_streams.insert(stream_tag, handle);
            }

            "unsubscribe" => {
                if let Some(handle) = active_streams.remove(&tag) {
                    handle.abort();
                    let _ = out_tx.send(json!({ "event": "unsubscribed", "tag": tag }).to_string());
                } else {
                    let _ = out_tx.send(json!({ "event": "error", "tag": tag, "error": "No active stream found with this tag" }).to_string());
                }
            }

            "overview" => {
                let st_clone = st.clone();
                let out_tx_clone = out_tx.clone();
                let tag_clone = tag.clone();

                tokio::spawn(async move {
                    let client_res = st_clone.resolve_client(req.router.as_ref(), req.router_id.as_deref()).await;
                    let client = match client_res {
                        Ok(c) => c,
                        Err(e) => {
                            let _ = out_tx_clone.send(json!({ "event": "overview_error", "tag": tag_clone, "error": e.to_string() }).to_string());
                            return;
                        }
                    };

                    let res_fut = client.run(build_command("/system/resource/print", std::iter::empty::<(&str, &str)>()));
                    let id_fut = client.run(build_command("/system/identity/print", std::iter::empty::<(&str, &str)>()));
                    let hs_fut = client.run(build_command("/ip/hotspot/active/print", std::iter::empty::<(&str, &str)>()));
                    let ppp_fut = client.run(build_command("/ppp/active/print", std::iter::empty::<(&str, &str)>()));

                    let (res, id, hs, ppp) = tokio::join!(res_fut, id_fut, hs_fut, ppp_fut);

                    let res_attrs = res.ok().and_then(|r| r.into_iter().next()).map(|r| r.attrs).unwrap_or_default();
                    let id_name = id.ok().and_then(|r| r.into_iter().next()).and_then(|r| r.get("name").map(str::to_owned)).unwrap_or_else(|| "MikroTik".into());
                    let hs_cnt = hs.map(|r| r.len()).unwrap_or(0);
                    let ppp_cnt = ppp.map(|r| r.len()).unwrap_or(0);

                    let _ = out_tx_clone.send(json!({
                        "event": "overview_result",
                        "tag": tag_clone,
                        "data": {
                            "identity": id_name,
                            "cpu_load": res_attrs.get("cpu-load").unwrap_or(&"0".into()),
                            "free_memory": res_attrs.get("free-memory").unwrap_or(&"0".into()),
                            "uptime": res_attrs.get("uptime").unwrap_or(&"".into()),
                            "version": res_attrs.get("version").unwrap_or(&"".into()),
                            "hotspot_active_count": hs_cnt,
                            "ppp_active_count": ppp_cnt
                        }
                    }).to_string());
                });
            }

            "listen_logs" => {
                let topic_filter = req.params.get("topic").cloned();
                let st_clone = st.clone();
                let out_tx_clone = out_tx.clone();
                let log_tag = tag.clone();

                if let Some(old) = active_streams.remove(&log_tag) {
                    old.abort();
                }

                let handle = tokio::spawn(async move {
                    let client_res = st_clone.resolve_client(req.router.as_ref(), req.router_id.as_deref()).await;
                    let client = match client_res {
                        Ok(c) => c,
                        Err(e) => {
                            let _ = out_tx_clone.send(json!({ "event": "stream_error", "tag": log_tag, "error": e.to_string() }).to_string());
                            return;
                        }
                    };

                    let mut args = vec![("follow-only", "")];
                    if let Some(t) = topic_filter.as_deref() {
                        args.push(("topics", t));
                    }
                    let sub_res = client.listen(build_command("/log/print", args)).await;
                    let mut sub = match sub_res {
                        Ok(s) => s,
                        Err(e) => {
                            let _ = out_tx_clone.send(json!({ "event": "stream_error", "tag": log_tag, "error": e.to_string() }).to_string());
                            return;
                        }
                    };

                    while let Some(item) = sub.next().await {
                        match item {
                            Ok(sentence) => {
                                let sent = out_tx_clone.send(json!({
                                    "event": "log_entry",
                                    "tag": log_tag,
                                    "data": sentence.attrs
                                }).to_string());
                                if sent.is_err() {
                                    break;
                                }
                            }
                            Err(_) => break,
                        }
                    }
                });

                active_streams.insert(log_tag, handle);
            }

            other => {
                let _ = out_tx.send(json!({ "event": "error", "tag": tag, "error": format!("Unknown action '{other}'") }).to_string());
            }
        }
    }

    // Clean up all active streams on websocket disconnect
    for (_, handle) in active_streams {
        handle.abort();
    }
    send_task.abort();
}

fn chrono_now_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}
