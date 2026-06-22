//! Server-Sent Events (SSE) live stream: `GET /events` (text/event-stream).
//!
//! Holds one TCP socket open for the life of a dashboard page and pushes a JSON
//! diff whenever state changes (temperature, LED state, link), plus a keepalive
//! comment every ~15 s so proxies/browsers don't drop the idle connection. The
//! browser falls back to polling /state if SSE is unavailable. The OTHER web pool
//! instance keeps serving normal requests while this socket is parked here.
//!
//! Wire format (one event):  `data: {"temp":23.5,"led":"FastBlink","link":1}\n\n`
//! Keepalive:                `: keepalive\n\n`

use embassy_net::{Stack, tcp::TcpSocket};

/// Stream events until the client disconnects. Reads temperature from
/// `crate::sensor::latest_temp()` and LED state from `crate::leds::STATE`.
pub async fn serve_events(socket: &mut TcpSocket<'_>, stack: Stack<'static>) {
    let _ = (socket, stack);
    // TODO: send 200 + text/event-stream headers. Loop: build JSON of
    // temp/led/link; on change write a `data: ...` event (two newlines); every
    // 15 s write a `: keepalive` comment; break on write error (client gone).
    todo!("stream SSE events until the client disconnects")
}
