use futures::{self, future, lazy, Async, Future, Poll};
use sql::prelude::*;
use std::io::{Error, ErrorKind};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use tokio_threadpool::ThreadPool;

// A Connection to query from a very slow database.
struct Connection;

impl Connection {
    pub fn new() -> Connection {
        Connection {}
    }

    // Our queries take about five seconds, our clients say.
    pub fn query(&self, q: &str) -> Result<(), Error> {
        thread::sleep(Duration::new(5, 0));
        Ok(println!("{}", q))
    }
}

// A very simple connection pool
struct Pool {
    connections: Vec<Connection>,
}

impl Pool {
    fn new() -> Pool {
        Pool {
            connections: vec![Connection::new()],
        }
    }

    // Get a connection, if available
    fn get(&self) -> Result<&Connection, Error> {
        self.connections
            .first()
            .ok_or(Error::new(ErrorKind::Other, "OH NO!"))
    }
}

// Database handling code in Prisma, containing pool and giving us a query
// method.
struct Database {
    pool: Pool,
}

impl Database {
    pub fn new() -> Database {
        Database { pool: Pool::new() }
    }

    pub fn query(&self, query: SelectQuery) -> Result<(), Error> {
        let conn = self.pool.get()?;
        let query = select_from("table")
            .columns(&["foo", "bar"])
            .so_that(query.field.equals(query.value))
            .compile()
            .unwrap();
        conn.query(&query)?;

        Ok(())
    }
}

// Query the database where field equals value
#[derive(Clone)]
struct SelectQuery {
    field: String,
    value: String,
}

fn main() {
    let database = Arc::new(Database::new());

    let query = SelectQuery {
        field: String::from("foo"),
        value: String::from("bar"),
    };

    let thread_pool = ThreadPool::new();

    dbg!("Creating the first future");
    let query_clone_1 = query.clone();
    let db_clone_1 = database.clone();
    let handle1 = thread_pool.spawn_handle(lazy(move || db_clone_1.query(query_clone_1)));

    dbg!("Creating the second future");
    let query_clone_2 = query.clone();
    let db_clone_2 = database.clone();
    let handle2 = thread_pool.spawn_handle(lazy(move || db_clone_2.query(query_clone_2)));

    // let final_future = handle1.and_then(|_| handle2).and_then(|_| {
    //     dbg!("Futures are finished");
    //     future::ok(())
    // });

    // let final_handle = thread_pool.spawn_handle(final_future);

    // Block the current thread until all the futures are finished, then exit
    // and print a stacktrace if the execution had any problems.
    dbg!("Waiting for futures to finish...");
    thread_pool.shutdown_on_idle().wait().unwrap();
}
