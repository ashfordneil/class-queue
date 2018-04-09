use config::Config;

use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Mutex;

use bcrypt;

use harsh::Harsh;

use serde_json;

use websocket::message::OwnedMessage;

/// A single student that is waiting for help.
#[derive(Debug, Serialize, Clone)]
pub struct Student {
    #[serde(skip)]
    id: u64,
    being_seen: bool,
    name: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ClientMessage<'a> {
    /// Request for help, with the name given
    Await(&'a str),
    /// Cancel a request for help, using the ID that was provided
    Cancel(&'a str),
    /// Log in, using your password
    Authenticate(&'a str),
    /// Send an alert that you're about to visit the students, using the ID that was provided
    Visit(&'a str),
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", content = "data")]
pub enum ServerMessage {
    /// Sent after await or successful authenticate
    YourId(String),
    /// Sent after unsuccessful authentication, or attempt to cancel with invalid ID
    UnAuthorized,
    /// Sent to all parties when the queue changes
    NewQueue(Vec<Student>),
    /// Sent to a student that is waiting when it is their turn to receive help - filtered out by
    /// the client not the students
    NewTurn,
}

#[derive(Debug, Clone)]
pub enum InternalMessage {
    NewQueue(Vec<Student>),
    NewTurn,
}

impl<'a> From<InternalMessage> for ServerMessage {
    fn from(other: InternalMessage) -> Self {
        match other {
            InternalMessage::NewQueue(data) => ServerMessage::NewQueue(data),
            InternalMessage::NewTurn => ServerMessage::NewTurn,
        }
    }
}

/// The state of the application
#[derive(Debug)]
pub struct State {
    /// Queue of students currently waiting for help
    queue: Mutex<VecDeque<Student>>,
    /// The bcrypt hash of the admin password
    password: String,
    /// The hasher to convert incremental login token IDs to random strings
    hasher: Harsh,
    /// List of all admin tokens
    admins: Mutex<Vec<u64>>,
    /// Largest allocated token + 1 (the next token to be allocated)
    max_token: Mutex<u64>,
    /// The root directory to serve static files from
    pub root_dir: PathBuf,
}

impl State {
    /// Create a new application state - reads the password from STDIN and panics on failure
    pub fn new(cfg: Config) -> Self {
        let queue = Mutex::new(VecDeque::new());

        let Config { root_dir, bcrypt_password: password, hasher } = cfg;

        let admins = Mutex::new(Vec::new());

        let max_token = Mutex::new(0);

        State {
            queue,
            password,
            hasher,
            admins,
            max_token,
            root_dir,
        }
    }

    /// Get the first message to send to a new client
    pub fn connect(&self) -> OwnedMessage {
        let queue = self.queue.lock().unwrap();
        let students = queue.iter().cloned().collect::<Vec<_>>();

        OwnedMessage::Text(serde_json::to_string(&ServerMessage::NewQueue(students)).unwrap())
    }

    /// Handles a client message, and returns optionally a message to be sent back to that client,
    /// and a message to be sent to the internal channel
    pub fn from_client<'a>(
        &self,
        message: ClientMessage<'a>,
    ) -> (Option<OwnedMessage>, Option<InternalMessage>) {
        lazy_static! {
            static ref NOPE: OwnedMessage =
                OwnedMessage::Text(serde_json::to_string(&ServerMessage::UnAuthorized).unwrap());
        }

        match message {
            ClientMessage::Await(name) => {
                // calculate the new student's details
                let id = {
                    let mut max_token = self.max_token.lock().unwrap();
                    let id = *max_token;
                    *max_token += 1;
                    id
                };
                let student = Student {
                    id,
                    being_seen: false,
                    name: name.into(),
                };

                // add them to the queue, and tell everyone
                let internal = {
                    let mut queue = self.queue.lock().unwrap();
                    queue.push_back(student);
                    InternalMessage::NewQueue(queue.iter().cloned().collect::<Vec<_>>())
                };

                // let the student know their ID
                let reply = ServerMessage::YourId(self.hasher.encode(&[id]).unwrap());

                info!("New student awaiting {}", name);

                // tell everyone about the updated queue
                (
                    Some(OwnedMessage::Text(serde_json::to_string(&reply).unwrap())),
                    Some(internal),
                )
            }
            ClientMessage::Cancel(id) => {
                info!("Await cancelled for {}", id);
                if let Some(id) = self.hasher.decode(id).map(|id| id[0]) {
                    let mut queue = self.queue.lock().unwrap();
                    queue.retain(|student| student.id != id);
                    (
                        None,
                        Some(InternalMessage::NewQueue(
                            queue.iter().cloned().collect::<Vec<_>>(),
                        )),
                    )
                } else {
                    (Some(NOPE.clone()), None)
                }
            }
            ClientMessage::Authenticate(password) => {
                let reply = if bcrypt::verify(&password, &self.password).unwrap_or(false) {
                    let id = {
                        let mut max_token = self.max_token.lock().unwrap();
                        let id = *max_token;
                        *max_token += 1;
                        id
                    };
                    {
                        let mut admins = self.admins.lock().unwrap();
                        admins.push(id);
                    }

                    info!("Successful login attempt");
                    OwnedMessage::Text(
                        serde_json::to_string(&ServerMessage::YourId(
                            self.hasher.encode(&[id]).unwrap(),
                        )).unwrap(),
                    )
                } else {
                    info!("Unsuccessful login attempt");
                    NOPE.clone()
                };

                (Some(reply), None)
            }
            ClientMessage::Visit(id) => {
                info!("New student being visited");
                if let Some(id) = self.hasher.decode(id).map(|id| id[0]) {
                    let approved = {
                        let admins = self.admins.lock().unwrap();
                        admins.contains(&id)
                    };

                    if approved {
                        let mut queue = self.queue.lock().unwrap();
                        if let Some(ref mut student) = queue.front_mut() {
                            student.being_seen = true;
                        }

                        (None, Some(InternalMessage::NewTurn))
                    } else {
                        (Some(NOPE.clone()), None)
                    }
                } else {
                    (Some(NOPE.clone()), None)
                }
            }
        }
    }
}
