use tokio::net::TcpStream;
use tokio_tungstenite::WebSocketStream;
use crate::services::poll_message;
use tokio::time::{timeout, Duration};

pub async fn check_auth_token(ws_stream: &mut WebSocketStream<TcpStream>) -> Result<(), String> {
    let timeout_duration = Duration::from_secs(5);
    let auth_token = match timeout(timeout_duration, poll_message::poll_text_message(ws_stream)).await {
        Ok(auth_token_result) => match auth_token_result {
            Ok(s) => s,
            Err(e) => {
                let error_message = format!("Error during poll_auth_token: {e}");
                return Err(error_message);
            }
        },
        Err(_) => {
            let error_message = String::from("poll_auth_token timeout");
            return Err(error_message);
        }
    };
    if auth_token != "123456" {
        let error_message = String::from("invalid auth_token");
        return Err(error_message);
    }
    Ok(())
}
