use rusqlite::{Connection, Result};
use std::time::Duration;

use crate::utils::{Event};

use std::collections::{HashMap, HashSet};

use log::{info,error};


// Database Design ////////
//
// Table: events
// Columns: 
//    - PK : event_id (hash of [movie_name,venue_code,date_string])
//    - movie_name
//    - venue_code (UPPERCASE String)
//    - date_string (YYYY-MM-DD)
//
// Table: event_users
//  Stores chat id and event it is waiting for
//  Columns:
//    - FK : event_id (hash of [movie_name,venue_code,date_string])
//    - chat_id
//      PK : (chat_id, event_id)
//
// Table: venues
//  Stores venue code and name
//  Columns:
//    - PK : venue_code (Uppercase String)
//    - venue_name
////////////////////////////


pub struct DbHelper{
    conn: Connection
}

impl DbHelper{
    pub fn new() -> Self{
        let conn = Connection::open("db/bms.db").unwrap();

        // The handler will sleep multiple times for sum total of 5 seconds
        // when SQLITE_BUSY error is returned. This happens when db is opened 
        // by another process.
        conn.busy_timeout(Duration::new(5, 0)).unwrap();

        // Initialize tables if not exist
        Self::init_tables(&conn).unwrap();

        Self{
            conn
        }
    }

    pub fn insert_event(&self, event: &Event) -> Result<usize>{
        info!("insert_event: {:?}", event);
        let event_hash = event.hash();
        let mut stmt = self.conn.prepare("INSERT OR IGNORE INTO events (
            event_id, movie_name, venue_code, date_string
        ) VALUES (?,?,?,?)").unwrap();
        stmt.execute([event_hash, 
            event.movie_name.to_string(), 
            event.venue_code.to_string(), 
            event.date_string.to_string()])
    }

    pub fn remove_event(&self, event: &Event) -> Result<usize>{
        info!("remove_event: {:?}", event);
        let event_hash = event.hash();

        // Delete corresponding event_user entries
        let mut stmt = self.conn.prepare("DELETE FROM event_users WHERE event_id = ?").unwrap();
        let res1 = stmt.execute([&event_hash]);
        if res1.is_err(){
            error!("Error deleting users for event: {:?}, Error : {:?}", event, res1);
        }

        // Remove the event
        stmt = self.conn.prepare("DELETE FROM events WHERE event_id = ?").unwrap();
        let res = stmt.execute([&event_hash]);
        if res.is_err(){
            error!("Error deleting event_id: {}, Error : {:?}", event_hash, res);
        }
        res
    }

    pub fn insert_user(&self, chat_id: i64, event: &Event) -> Result<usize>{
        info!("insert_user: chat_id: {}, event: {:?}", chat_id, event);
        let event_hash = event.hash();
        // Add event if not present
        self.insert_event(event)?;
        let mut stmt = self.conn.prepare("INSERT OR IGNORE INTO event_users (
            event_id, chat_id
        ) VALUES (?,?)").unwrap();
        stmt.execute([event_hash, chat_id.to_string()])
    }

    pub fn get_all_events_and_users(&self) -> Result<HashMap<Event, HashSet<i64>>>{
        info!("Getting all events and users");
        let mut stmt = self.conn.prepare("SELECT * FROM events").unwrap();

        let events = stmt.query_map([], |row| {
            let movie_name: String = row.get(1).unwrap();
            let venue_code: String = row.get(2).unwrap();
            let date_string: String = row.get(3).unwrap();
            Ok(Event::new(movie_name, venue_code, date_string))
        })?;

        let mut result = HashMap::new();
        for event in events{
            let event = event.unwrap();
            let mut stmt = self.conn.prepare("SELECT chat_id FROM event_users WHERE event_id = ?").unwrap();
            let user_ids = stmt.query_map([&event.hash()], |row| row.get(0))?;
            let mut user_set = HashSet::new();
            for user_id in user_ids{
                user_set.insert(user_id.unwrap());
            }
            result.insert(event, user_set);
        }
        Ok(result)
    }

    pub fn get_events_for_user(&self, chat_id: i64) -> Result<Vec<Event>>{
        info!("Getting events for user: {}", chat_id);
        let mut stmt = self.conn.prepare("SELECT e.* from events e, event_users u WHERE u.chat_id = ? AND e.event_id = u.event_id;").unwrap();
        let events = stmt.query_map([&chat_id], |row| {
            let movie_name: String = row.get(1).unwrap();
            let venue_code: String = row.get(2).unwrap();
            let date_string: String = row.get(3).unwrap();
            Ok(Event::new(movie_name, venue_code, date_string))
        })?.collect::<Result<Vec<Event>>>()?;
        Ok(events)
    }


    pub fn get_waiting_users(&self, event: &Event) -> Result<Vec<i64>>{
        info!("Getting waiting users for event {:?}", event);
        let event_hash = event.hash();
        let mut stmt = self.conn.prepare("SELECT chat_id FROM event_users WHERE event_id = ?").unwrap();
        let mut users = Vec::new();
        for user_id in stmt.query_map([&event_hash], |row| row.get(0))?{
            users.push(user_id.unwrap());
        }
        Ok(users)
    }

    pub fn insert_venue(&self, venue_code: &str, venue_name: &str) -> Result<usize>{
        info!("insert_venue: {:?}, {:?}", venue_code, venue_name);
        let mut stmt = self.conn.prepare("REPLACE INTO venues (venue_code, venue_name) VALUES (?,?)").unwrap();
        stmt.execute([venue_code, venue_name])
    }

    pub fn get_venue_name(&self, venue_code: &str) -> Result<String>{
        info!("get_venue_name: {:?}", venue_code);
        let mut stmt = self.conn.prepare("SELECT venue_name FROM venues WHERE venue_code = ?").unwrap();
        stmt.query_row([venue_code], |row| row.get(0))
    }


    fn init_tables(conn: &Connection) -> Result<()>{
        info!("Initializing tables");

        conn.execute("CREATE TABLE IF NOT EXISTS events (
            event_id TEXT PRIMARY KEY,
            movie_name TEXT NOT NULL,
            venue_code TEXT NOT NULL,
            date_string TEXT NOT NULL
        )", [])?;

        conn.execute("CREATE TABLE IF NOT EXISTS event_users (
            event_id TEXT NOT NULL,
            chat_id INTEGER NOT NULL,
            FOREIGN KEY (event_id) REFERENCES events(event_id),
            PRIMARY KEY (event_id, chat_id)
        )", [])?;

        conn.execute("CREATE TABLE IF NOT EXISTS venues (
            venue_code TEXT PRIMARY KEY,
            venue_name TEXT NOT NULL
        )", [])?;
        Ok(())
    }
}