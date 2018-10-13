extern crate dotenv;
extern crate pretty_env_logger;
extern crate warp;

use warp::{Filter, Future, Stream};

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
                rx.forward(tx)
                    .map(|_| ())
                    .map_err(|e| {
                        eprintln!("websocket error: {:?}", e);
                    })
            })
        });
    let routes = elm
        .or(echo);

    let port: u16 = std::env::var("PORT").map_err(|_| ())
        .and_then(|s| <u16 as std::str::FromStr>::from_str(&s).map_err(|_| ()))
        .unwrap_or(3030);
    warp::serve(routes)
        .run(([127, 0, 0, 1], port));
}
