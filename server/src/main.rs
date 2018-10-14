extern crate dotenv;
extern crate pretty_env_logger;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate warp;

use warp::{Filter, Future, Stream};
use warp::filters::ws::Message;

#[derive(Deserialize)]
#[serde(tag = "type")]
enum ClientCommand {
    Shout { message: String },
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum ServerCommand {
    Echo { message: String },
    Timestamp { when: i64 },
}

struct EchoError;

// TODO: use error-chain
impl std::convert::From<()> for EchoError {

    fn from(_: ()) -> EchoError {
        EchoError
    }

}

impl std::convert::From<serde_json::Error> for EchoError {

    fn from(_: serde_json::Error) -> EchoError {
        EchoError
    }

}

fn echo_err(message: Message) -> Result<Message, EchoError> {
    let message_text = message.to_str()?;
    let command: ClientCommand = serde_json::from_str(message_text)?;
    let response = match command {
        ClientCommand::Shout { message } => ServerCommand::Echo { message: message },
    };
    let response_text = serde_json::to_string(&response)?;
    Ok(Message::text(response_text))
}

fn echo(message: Message) -> Message {
    echo_err(message).unwrap_or(Message::text(serde_json::to_string(&ServerCommand::Timestamp { when: 0 }).unwrap()))
}

fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::init();

    let elm = warp::fs::dir("../web");
    let echo = warp::path("echo")
        // The `ws2()` filter will prepare the Websocket handshake.
        .and(warp::ws2())
        .map(|ws: warp::ws::Ws2| {
            // And then our closure will be called when it completes...
            ws.on_upgrade(|websocket| {
                // Just echo all messages back...
                let (tx, rx) = websocket.split();
                rx
                    .map(echo)
                    .forward(tx)
                    .map(|_| ())
                    .map_err(|e| {
                        eprintln!("websocket error: {:?}", e);
                    })
            })
        });
    let routes = echo
        .or(elm);

    let port: u16 = std::env::var("PORT").map_err(|_| ())
        .and_then(|s| <u16 as std::str::FromStr>::from_str(&s).map_err(|_| ()))
        .unwrap_or(3030);
    warp::serve(routes)
        .run(([0, 0, 0, 0], port));
}
