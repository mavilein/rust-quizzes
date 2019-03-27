use futures::{self, future, Async, Future, Poll};
use sql::prelude::*;
use std::io::{Error, ErrorKind};
use std::thread;
use std::time::Duration;
use tokio::runtime::Runtime;
use tower_service::Service;

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
    let mut database = Database::new();

    let query = SelectQuery {
        field: String::from("foo"),
        value: String::from("bar"),
    };

    let mut rt = Runtime::new().unwrap();

    // Create our two database calls, the dumb implementation will do a
    // blocking query and this actually does not help to make our system
    // faster.
    //
    // The calls here go directly to the runtime, which will start executing
    // them in a separate thread.
    dbg!("Creating the first future");
    rt.spawn(database.query(query.clone()));
    dbg!("Creating the second future");
    rt.spawn(database.query(query.clone()));

    // Block the current thread until all the futures are finished, then exit
    // and print a stacktrace if the execution had any problems.
    dbg!("Waiting for futures to finish...");
    rt.shutdown_on_idle().wait().unwrap();
}
