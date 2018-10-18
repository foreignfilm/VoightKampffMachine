extern crate dotenv;
extern crate futures;
#[macro_use] extern crate log;
extern crate pretty_env_logger;
extern crate rand;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate warp;

use futures::Sink;
use futures::stream::SplitSink;
use futures::sink::Wait;
use std::collections::HashMap;
use std::sync::{ Arc, Mutex };
use warp::{ Filter, Future, Stream };
use warp::filters::ws::{ Message, WebSocket };

#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
struct SuspectId(String);

#[derive(Clone, Debug, Copy, Eq, Hash, PartialEq, Serialize, Deserialize)]
struct ConnectionId(u64);

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum ClientCommand {
    LogInAsSuspect,
    LogInAsInvestigator { suspect_id: SuspectId },
    InvestigatorShout { message: String },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
enum ServerCommand {
    Connected,
    BecomeSuspect { suspect_id: SuspectId },
    BecomeInvestigator { suspect_id: SuspectId },
    Echo { message: String },
}

struct Game {
    suspect_cid: ConnectionId,
    investigator_cid: Option<ConnectionId>,
}

struct Bureau {
    next_cid: ConnectionId,
    connections: HashMap<ConnectionId, Wait<SplitSink<WebSocket>>>,
    games: HashMap<SuspectId, Game>,
}

const SUSPECT_ID_LETTERS: &[u8] = b"BCDFGHJKLMNPQRSTVWXZ";

impl Bureau {

    fn new() -> Bureau {
        Bureau {
            next_cid: ConnectionId(0),
            connections: HashMap::new(),
            games: HashMap::new(),
        }
    }

    fn next_cid(&mut self) -> ConnectionId {
        let result = self.next_cid;
        self.next_cid = ConnectionId(result.0 + 1);
        result
    }

    fn connect(&mut self, connection: Wait<SplitSink<WebSocket>>) -> ConnectionId {
        let cid = self.next_cid();
        let old_value = self.connections.insert(cid, connection);
        assert!(old_value.is_none());
        cid
    }

    fn disconnect(&mut self, cid: ConnectionId) {
        // TODO: what about games in progress?
        self.connections.remove(&cid);
    }

    fn send(
        &mut self,
        cid: ConnectionId,
        command: ServerCommand
    ) -> Result<(), InhumanityError> {
        let command_text = serde_json::to_string(&command)?;
        let connection = self.connections.get_mut(&cid).ok_or(InhumanityError)?;
        connection.send(Message::text(command_text))?;
        connection.flush()?;
        Ok(())
    }

    fn next_suspect_id(&self) -> SuspectId {
        let suspect_id = SuspectId(String::from_utf8(rand::seq::sample_slice(
            &mut rand::thread_rng(),
            SUSPECT_ID_LETTERS,
            4)).unwrap());
        if !self.games.contains_key(&suspect_id) {
            suspect_id
        } else {
            self.next_suspect_id()
        }
    }

    fn new_game(&mut self, suspect_cid: ConnectionId) -> SuspectId {
        let suspect_id = self.next_suspect_id();
        self.games.insert(suspect_id.clone(), Game {
            suspect_cid: suspect_cid,
            investigator_cid: None,
        });
        suspect_id
    }

    // FIXME: this is causing minor lifetime issues, rethink?
    fn game(&mut self, suspect_id: &SuspectId) -> Option<&mut Game> {
        self.games.get_mut(suspect_id)
    }

}

// TODO: don't just shove everything behind one giant mutex, actually attempt
// to scale.
type DB = Arc<Mutex<Bureau>>;

struct InhumanityError;

// TODO: use error-chain
macro_rules! convert_inhumanity {
    ( $from:ty ) => {
        impl std::convert::From<$from> for InhumanityError {

            fn from(_: $from) -> InhumanityError {
                InhumanityError
            }

        }
    };
    ( $t:ident, $from:ty ) => {
        impl<$t> std::convert::From<$from> for InhumanityError {

            fn from(_: $from) -> InhumanityError {
                InhumanityError
            }

        }
    };
}

convert_inhumanity!(());
convert_inhumanity!(serde_json::Error);
convert_inhumanity!(warp::Error);
convert_inhumanity!(std::string::FromUtf8Error);
convert_inhumanity!(T, std::sync::PoisonError<T>);

fn handle_client_message(
    db: DB,
    cid: ConnectionId,
    suspect_id: Option<SuspectId>,
    message: Message
) -> Result<Option<SuspectId>, InhumanityError> {
    let message_text = message.to_str()?;
    let command: ClientCommand = serde_json::from_str(message_text)?;
    use ClientCommand::*;
    // TODO: check state for state machine!
    trace!("received {:?}", command);
    match command {
        LogInAsSuspect => {
            let mut bureau = db.lock()?;
            let suspect_id = bureau.new_game(cid);
            trace!("assigned suspect_id {:?} to cid {:?}", suspect_id, cid);
            bureau.send(cid, ServerCommand::BecomeSuspect {
                suspect_id: suspect_id.clone()
            })?;
            Ok(Some(suspect_id))
        },
        LogInAsInvestigator { suspect_id } => {
            let mut bureau = db.lock()?;
            {
                let game = bureau.game(&suspect_id).ok_or(InhumanityError)?;
                if game.investigator_cid.is_some() {
                    // tried to connect to a game with an investigator already
                    return Err(InhumanityError)
                }
                trace!("{:?} now investigating {:?}", cid, suspect_id);
                game.investigator_cid = Some(cid);
            }
            bureau.send(cid, ServerCommand::BecomeInvestigator {
                suspect_id: suspect_id.clone()
            })?;
            Ok(Some(suspect_id))
        },
        InvestigatorShout { message } => {
            let mut bureau = db.lock()?;
            let suspect_id = suspect_id.ok_or(InhumanityError)?;
            let (investigator_cid, suspect_cid) = {
                let game = bureau.game(&suspect_id).ok_or(InhumanityError)?;
                (game.investigator_cid.ok_or(InhumanityError)?, game.suspect_cid)
            };
            if investigator_cid == cid {
                trace!("investigator of {:?} yells {:?} at {:?}", suspect_id, message, suspect_cid);
                bureau.send(suspect_cid, ServerCommand::Echo {
                    message: message
                })?;
                Ok(Some(suspect_id))
            } else {
                // tried to shout as suspect
                Err(InhumanityError)
            }
        },
    }
}

fn main() {
    dotenv::dotenv().ok();
    pretty_env_logger::init();

    let storage: DB = Arc::new(Mutex::new(Bureau::new()));
    let db = warp::any().map(move || storage.clone());

    let elm = warp::fs::dir("../web");
    let inhumanity = warp::path("inhumanity")
        .and(db.clone())
        .and(warp::ws2())
        .map(|db: DB, ws: warp::ws::Ws2| {
            ws.on_upgrade(move |websocket| {
                let (tx, rx) = websocket.split();
                let db1 = db.clone();
                let db2 = db.clone();
                let db3 = db.clone(); // gross
                futures::future::ok(())
                    .and_then(move |_: ()| -> Result<ConnectionId, InhumanityError> {
                        let mut bureau = db1.lock()?;
                        let cid = bureau.connect(tx.wait());
                        bureau.send(cid, ServerCommand::Connected)?;
                        trace!("connected as {:?}", cid);
                        Ok(cid)
                    })
                    .and_then(move |cid| {
                        rx
                            .map_err(move |e| {
                                eprintln!("websocket error: {:?}", e);
                                db2.lock().ok().map(|mut bureau| bureau.disconnect(cid));
                                InhumanityError
                            })
                            .fold(None, move |suspect_id, message| {
                                handle_client_message(db3.clone(), cid, suspect_id, message)
                            })
                            .map(move |_| {
                                trace!("{:?} disconnected (done)", cid);
                            })
                            .map_err(move |e| {
                                trace!("{:?} disconnected (error)", cid);
                                e
                            })
                    })
                    .map(|_| {
                        trace!("finito (done)");
                    })
                    .map_err(move |_| {
                        trace!("finito (error)");
                    })
            })
        });
    let routes = inhumanity.or(elm);

    let port: u16 = std::env::var("PORT").map_err(|_| ())
        .and_then(|s| <u16 as std::str::FromStr>::from_str(&s).map_err(|_| ()))
        .unwrap_or(3030);
    warp::serve(routes)
        .run(([0, 0, 0, 0], port));
}
