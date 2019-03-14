mod db {
    //! Do not change anything inside the `db` module

    use std::collections::VecDeque;

    /// Stores some data as a queue
    pub struct Database {
        store: VecDeque<String>,
    }

    /// Commands supported by the database
    #[derive(Eq, PartialEq, Debug)]
    pub enum Command {
        Pub(String),
        Get,
    }

    #[derive(Eq, PartialEq, Debug)]
    pub enum Error {
        UnnownCmd,
        BadPayload,
        Incomplete,
    }

    pub fn parse(input: &str) -> Result<Command, Error> {
        let mut split = input.splitn(2, ' ');

        if let Some(verb) = split.next() {
            match verb.trim() {
                "GET" => {
                    if split.next() == None {
                        Ok(Command::Get)
                    } else {
                        Err(Error::BadPayload)
                    }
                }
                "PUB" => {
                    if let Some(payload) = split.next() {
                        Ok(Command::Pub(payload.trim().into()))
                    } else {
                        Err(Error::BadPayload)
                    }
                }
                "" => Err(Error::Incomplete),
                _ => Err(Error::UnnownCmd),
            }
        } else {
            Err(Error::Incomplete)
        }
    }

    impl Database {
        pub fn new() -> Self {
            Self {
                store: VecDeque::new(),
            }
        }

        pub fn get(&mut self) -> Option<String> {
            self.store.pop_back()
        }

        pub fn store(&mut self, msg: String) {
            self.store.push_back(msg);
        }
    }
}

mod tp {
    //! Some things need implementing!
    use std::sync::mpsc;
    use std::sync::mpsc::{Receiver, Sender};
    use std::thread::{spawn, JoinHandle};

    pub struct ThreadPool {
        // TODO: Store the threads somehow
        round_robin_counter: usize,
        channels: Vec<Sender<Job>>,
    }

    impl ThreadPool {
        pub fn new(size: usize) -> Self {
            let mut channels = vec![];
            for i in 0..size {
                // join_handles.push(spawn)
                let (tx, rx) = mpsc::channel();
                let worker = Worker::new(i, rx);
                channels.push(tx);
                spawn(move || {
                    worker.start();
                });
            }
            Self {
                // TODO: start the threads somehow
                round_robin_counter: 0,
                channels: channels,
            }
        }

        /// Queue some work to be run
        pub fn queue<F>(&mut self, f: F)
        where
            F: FnOnce() + Send + 'static,
        {
            let job = Box::new(f);
            let worker = self.channels.get(self.round_robin_counter).unwrap();
            worker.send(job).unwrap();
            // TODO: this must round rob actually
            self.round_robin_counter = self.round_robin_counter + 1;
        }
    }

    trait FnBox {
        fn call_box(self: Box<Self>);
    }

    impl<F: FnOnce()> FnBox for F {
        fn call_box(self: Box<F>) {
            (*self)()
        }
    }

    /// An easy to use type that wraps around a static function
    /// that is only called once, passed between threads.
    ///
    /// This acts as the main Job type for our thread pool
    type Job = Box<FnBox + Send + 'static>;

    struct Worker {
        _id: usize,
        // _thread: JoinHandle<()>,
        receiver: Receiver<Job>,
    }

    impl Worker {
        // FIXME: why the comment block after parameter?
        fn new(_id: usize, receiver: Receiver<Job>) -> Self {
            // TODO: Start a worker that waits for work
            // unimplemented!()
            println!("Worker created(id: {})", _id);
            Worker { _id, receiver }
        }

        fn start(&self) {
            let job = self.receiver.recv().unwrap();
            println!("Worker.start(id: {})", self._id);
            job.call_box()
        }
    }

}
use std::io::prelude::*;
use std::io::BufReader;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

use db::{Command, Database};
use tp::ThreadPool;

use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread::{spawn, JoinHandle};

// trait AsyncDatabase {
//     fn get(callback: Sender<Option<String>>);
//     fn store(msg: String, callback: Sender<()>);
// }

#[derive(Clone)]
struct AsyncDatabaseImpl {
    sender: Sender<AsyncDatabaseCommand>,
}

#[derive(Debug)]
enum AsyncDatabaseCommand {
    Get(Sender<Option<String>>),
    Store(String, Sender<()>),
}

impl AsyncDatabaseImpl {
    fn new() -> AsyncDatabaseImpl {
        let (sender, receiver) = mpsc::channel::<AsyncDatabaseCommand>();
        spawn(move || {
            let mut database = Database::new();
            let cmd = receiver.recv().unwrap();
            println!("AsyncDatabaseImpl received command: {:?}", cmd);
            match cmd {
                AsyncDatabaseCommand::Get(callback) => {
                    callback.send(database.get());
                }
                AsyncDatabaseCommand::Store(value, callback) => {
                    database.store(value);
                    callback.send(());
                }
            }
        });

        AsyncDatabaseImpl { sender }
    }

    fn get(&self, callback: Sender<Option<String>>) {
        self.sender.send(AsyncDatabaseCommand::Get(callback));
    }
    fn store(&self, value: String, callback: Sender<()>) {
        self.sender
            .send(AsyncDatabaseCommand::Store(value, callback));
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    // Store data
    // let database = Arc::new(Mutex::new(Database::new()));
    let async_database = AsyncDatabaseImpl::new();

    // Creates a threadpool with 4 connection workers
    let mut pool = ThreadPool::new(4);

    // This is an infinite iterator
    for stream in listener.incoming() {
        // let database_mutex = Arc::clone(&database);
        let async_database_clone = async_database.clone();
        pool.queue(move || {
            let mut stream = stream.unwrap();
            // FIXME: this loop will always exhaust one thread, right?
            loop {
                let mut read_buffer = String::new();
                let mut buffered_stream = BufReader::new(&stream);
                if let Err(_) = buffered_stream.read_line(&mut read_buffer) {
                    break;
                }

                let cmd = db::parse(&read_buffer);
                println!("got command {:?}", cmd);
                // let mut database = database_mutex.lock().unwrap();

                match cmd {
                    Ok(Command::Get) => {
                        let (sender, receiver) = mpsc::channel::<Option<String>>();
                        async_database_clone.get(sender);
                        let result = receiver.recv().unwrap();
                        send_reply(&mut stream, result.unwrap_or_else(|| "<empty>".into()));
                    }
                    Ok(Command::Pub(s)) => {
                        let (sender, receiver) = mpsc::channel::<()>();
                        async_database_clone.store(s, sender);
                        receiver.recv().unwrap();
                        send_reply(&mut stream, "<done>");
                    }
                    Err(e) => send_reply(&mut stream, format!("<error: {:?}>", e)),
                }
            }
        });
    }
}

// No need to really touch this function
fn send_reply<'a, S: Into<String>>(stream: &mut TcpStream, msg: S) {
    // Sometimes we break and we don't care
    let _ = stream.write(msg.into().as_bytes());
    let _ = stream.write("\r\n".as_bytes());
}
