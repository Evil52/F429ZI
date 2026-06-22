//! HTTP dashboard server (raw smoltcp TcpSocket, no framework).
//!
//! Architecture copied from the JZF407 reference (it's proven on hardware):
//!   - TWO near-identical tasks (`web_task`, `web_task_b`) each own their own
//!     static TCP buffers. Both delegate to the SAME router (`serve_connection`),
//!     so either socket can serve a normal page OR hold the long-lived
//!     `GET /events` SSE stream. smoltcp hands accepted connections to whichever
//!     socket is free, so both must know every route (else random 404s).
//!   - Sockets come from `StackResources<N>` allocated in main.rs.
//!   - HTML is streamed from flash-resident &str chunks (no big RAM buffer).
//!   - Nagle disabled (many small writes per page + host delayed-ACK = slow page).
//!
//! Routes for THIS product (temperature + LEDs + config):
//!   GET  /          dashboard HTML
//!   GET  /state     JSON snapshot {temp, led_state, button, link, up}
//!   GET  /events    SSE live stream (see sse.rs)
//!   POST /led       set LED state (AllOff/SlowBlink/FastBlink/AllOn)
//!   POST /save      validate + write config to Flash, then reboot
//!   POST /reboot    reboot (fault::safe_reboot)
//!   POST /firmware  OTA: receive .bin into Slot B, verify, swap, reboot (ota.rs)
//!   POST /login /logout   session auth (logic::auth)
//!
//! Auth model = form-login + session cookie (NOT HTTP Basic — browsers cache
//! Basic creds so logout is impossible). Empty user+pass ⇒ auth disabled.

use embassy_net::Stack;
use f429zi_logic::config::Config;

use crate::fault::ResetReason;

/// HTTP server, pool instance A. Owns its own static buffers.
#[embassy_executor::task]
pub async fn web_task(stack: Stack<'static>, cfg: Config, reset: ResetReason) {
    let _ = (stack, cfg, reset);
    todo!(
        "wait stack.wait_link_up()/wait_config_up(); init RX/TX/REQ StaticCell bufs; \
         call serve_pool(...) — see reference web.rs"
    )
}

/// HTTP server, pool instance B. Identical to A but with its own buffers
/// (embassy tasks can't share a StaticCell, hence two task fns).
#[embassy_executor::task]
pub async fn web_task_b(stack: Stack<'static>, cfg: Config, reset: ResetReason) {
    let _ = (stack, cfg, reset);
    todo!("same as web_task with a separate set of StaticCell buffers")
}

// ── Below: the shared router skeleton you'll fill in. Kept as free fns (not in
//    the task) so both pool instances share exactly one implementation. ────────

// async fn serve_pool(stack, cfg, reset, rx_buf, tx_buf, req_buf) { accept loop }
// async fn serve_connection(socket, cfg, stack, reset, req_buf, auth...) { router }
// fn parse_form(body, current) -> Option<Config> { ... uses logic::parse }
// async fn send_page(socket, ...) { stream the dashboard, temp from sensor::latest_temp() }
// async fn send_state(socket, ...) { compact JSON for the poller }
