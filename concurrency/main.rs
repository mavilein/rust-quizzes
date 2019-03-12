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
    use std::thread::{spawn, JoinHandle};

    pub struct ThreadPool {
        // TODO: Store the threads somehow
        rrctr: usize,
    }

    impl ThreadPool {
        pub fn new(size: usize) -> Self {
            Self {
                rrctr: 0,
                // TODO: start the threads somehow
            }
        }

        /// Queue some work to be run
        pub fn queue<F>(&mut self, f: F)
        where
            F: FnOnce() + Send + 'static,
        {
            let job = Box::new(f);

            // TODO: Be a boss and assign jobs to workers
            unimplemented!()
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
        _thread: JoinHandle<()>,
    }

    impl Worker {
        fn new(_id: usize /* ... */) -> Self {
            // TODO: Start a worker that waits for work
            unimplemented!()
        }
    }

}

use std::io::prelude::*;
use std::io::BufReader;
use std::net::{TcpListener, TcpStream};

use db::{Command, Database};
use tp::ThreadPool;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();

    // Store data
    let storage = Database::new();

    // Creates a threadpool with 4 connection workers
    let mut pool = ThreadPool::new(4);

    // This is an infinite iterator
    for stream in listener.incoming() {
        pool.queue(move || {
            let mut stream = stream.unwrap();

            loop {
                let mut read_buffer = String::new();
                let mut buffered_stream = BufReader::new(&stream);
                if let Err(_) = buffered_stream.read_line(&mut read_buffer) {
                    break;
                }

                let cmd = db::parse(&read_buffer);

                match cmd {
                    Ok(Command::Get) => send_reply(
                        &mut stream,
                        storage.get().unwrap_or_else(|| "<empty>".into()),
                    ),
                    Ok(Command::Pub(s)) => {
                        storage.store(s);
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
