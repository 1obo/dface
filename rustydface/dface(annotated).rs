#![allow(unused)]

use std::error::Error;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Debug, Display};
use std::time::Duration;
use std::io::{Read, BufRead, BufReader};
use std::string::String;
use rusqlite::*;
use ssdeep::*;
use chrono::*;
use thirtyfour::prelude::*;
use image_hasher::{HasherConfig, Hasher, HashAlg};
use rusqlite::fallible_iterator::FallibleIterator;
use rusqlite::ffi::sqlite3_uri_boolean;
use tokio::main;


//ADD ATTRIBUTION TO DESTINATION. This means some sort of unit of distance to measure for this one.

struct Logging {
    timestamp: i64,
    uri: String,
    similarity: u32,
}
// This will need further development. In our use case, if it lives within the hive, perhaps writing this to a syslog format is the best way to continue. For now, this is simply there to complete our work flow
impl Logging {
    fn new(timestamp: i64, uri: String, similarity: u32) -> Logging {
        Logging {timestamp, uri, similarity }
    }
    
    
    fn check(logs:Logging, db: &Db, threshold:u32) {
        if logs.similarity < threshold {
            let foo = db.dbexecute("
            INSERT INTO logs (timestamp, uri, similarity, action) VALUES (?1, ?2, ?3, 'ALERT') ", 
    &[&logs.timestamp, &logs.uri, &logs.similarity]);}
        else if logs.similarity > 50 && logs.similarity < 70 {
            let foo = db.dbexecute("
            INSERT INTO logs (timestamp, uri, similarity, action) VALUES (?1, ?2, ?3, 'Log') ",
                &[&logs.timestamp, &logs.uri, &logs.similarity]);}
        else {}
    }
         
}




#[derive(Debug)]
struct Monitor {

    id: Option<i64>,
    uri: String,
    frequency: i64,
    threshold: u32
    //threshold: This will be a numeric value that is appended to each primary key in our table, the implementation for this will be passed into the alert object in our pages compare function, and we will then use it to determine the action regarding our event
      }

impl Monitor {
    //How do we look down newwithid.... perhaps logic in main if it can be kept somewhat idiomatic/natural?
    //Maybe a trait that could determine this logic for us?
    //Something like impl id for monitor where the Some or None determines which function entry we enter
    //If in main, something such is if None(id) { Monitor::new() else if Some(id) { Monitor::newwithid} else println!("Error: id parse failure")
    // The above does not seem very natural
    pub fn new(uri: &str, frequency: i64, threshold: u32) -> Monitor {
        Monitor { id:None, uri:uri.to_string(), frequency, threshold }
    }

    //Lock down newwithid to make it only accessible in the case that an ID is not present in the Db
    fn newwithid(id: i64, uri: &str, frequency: i64, threshold:u32) -> Monitor {
        Monitor { id:Some(id), uri:uri.to_string(), frequency, threshold }
    }
    
    fn getall(db: &Db) -> Result<Vec<Monitor>> {
        let mut stmt = db.conn.prepare(
            "SELECT id, uri, frequency, threshold FROM monitored"
        )?;
        //somewhere around here we need to add on operation checking 
        // if self.frequency>Local::now() { Create a new page object } 
            // else {}
        let destinations = stmt.query_map([], |row| {
            Ok(Monitor{
                id: Some(row.get(0)?),
                uri:row.get(1)?,
                frequency:row.get(2)?,
                threshold:row.get(3)?
            })
        })?
            
            .collect::<Result<Vec<Monitor>>>()?;
            Ok(destinations)
        //let time = destinations.into_iter().
        // if destinations.into_iter().any(|d| d.frequency>Local::now().timestamp()) {
        //     Ok(destinations)
        // }
        // else{
        //     None(destinations)
        // }
    }
    fn repr(destinations: Vec<Monitor>) -> Vec<String> {
        destinations.into_iter().map(|m| m.uri).collect()

    }
    // fn getouput(db: &Db, page: &Page) -> Vec<Monitor> {
    //
    //
    // }




    fn save(&mut self, db: &Db) {
        match &self.id {
            Some(id) => {
                let var = db.dbexecute(
                    "UPDATE monitored SET uri = ?1, frequency = ?2 WHERE id = ?3",
                &[&self.uri, &self.frequency, id]);

            }
            None => {
                let foo = db.dbexecute("INSERT INTO monitored (uri, frequency) VALUES (?1, ?2)",
                    &[&self.uri, &self.frequency]);
                let id = db.conn.last_insert_rowid();
                self.id = Some(id);
            }
        }
    }



}

impl Display for Monitor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "id:{:?}, uri:{:?}, frequency:{:?}", self.id, self.uri, self.frequency)
    }
}

struct Db {
    file: String,
    conn: Connection,
}


impl Db {
    fn new() -> Result<Db, Box<dyn Error>> {
        let file = "dface.sqlite".to_string();
        let conn = Connection::open(file.clone()).expect("sqlite open error");
        Ok(Db { file, conn })
    }

    fn dbexecute(&self, statement: &str, params: &[&dyn ToSql]) -> Result<()> {
        self.conn.execute(statement, params).expect("dbexecute error");
        Ok(())
        
    }
}


#[derive(Debug)]
struct Page {
    uri: Option<String>,
    html: String,
    // image: ,
    timestamp: i64,
    sshash: String,
    // phash: String,
    // dhash: String,

}

impl Page {
    #[tokio::main]
    async fn new(uri: &str) -> WebDriverResult<Self> {
        let capabilities = DesiredCapabilities::chrome();

        let driver = WebDriver::new("http://localhost:61015", capabilities).await.expect("Failed to connect to WebDriver");
        driver.set_window_rect(1, 1, 1920, 1080);
        driver.goto(uri).await.expect("Failed to get URI from WebDriver");
        let html = driver.source().await.expect("Failed to get HTML from WebDriver");
        let image = driver.screenshot_as_png().await.expect("Failed to get image from WebDriver");
        let timestamp = Local::now().timestamp();
        driver.quit().await;
        let sshash = Self::hashvalues(&html).await;
        Ok(
            Page
            {
                uri: Some(uri.to_string()),
                html,
                timestamp,
                sshash,
                // phash,
                // dhash
            })
    }


    async fn hashvalues(html: &str) -> String {
        let sshash1 = hash_buf(html.as_bytes());
        sshash1.unwrap().to_string()
    }
    fn getall(db: &Db) -> Result<Vec<Page>> {
        let mut stmt = db.conn.prepare(
            "SELECT uri, html, timestamp, sshash FROM pagesample"
        )?;
        let sources = stmt.query_map([], |row| {
            Ok(Page{
                uri: Some(row.get(0)?),
                html: row.get(1)?,
                sshash: row.get(3)?,
                timestamp: row.get(6)?
            })
        })?
        .collect::<Result<Vec<Page>>>()?;
        Ok(sources)

    }
    // This function handles all monitored uri's that are not present within our assigned frequency range, and updates them with the newest associated values.
    // fn refresh(monitor: &Monitor, db: &Db) -> () {
    //     let mut stmt1 = db.conn.prepare("
    //         SELECT uri, html, timestamp, sshash FROM pagesample WHERE uri = ?1 AND timestamp >= ?2
    //         ORDER BY timestamp DESC LIMIT 1").expect("Could not prepare query");
    //     let oldest_monitor = Utc::now().timestamp() - monitor.frequency;
    //     let page = stmt1.query_row([&monitor.uri, &oldest_monitor.to_string()], |row| {
    // 
    //         Ok(Page {
    //             uri: row.get(0)?,
    //             html: row.get(1)?,
    //             sshash: row.get(3)?,
    //             timestamp: row.get(2)?
    // 
    //         })
    // 
    // 
    //     });
    //     match page {
    //         Ok(page) => (),
    //         Err(_) => Page::new(&monitor.uri).expect("Could not create page").save(&db),
    //         }
    //     
    // }
    // This function handles all monitored uri's that are not present within our assigned frequency range, creates a new page for the expired value, and performs a hash comparison on the old object, and the newly acquired hash value. This comparison is a score ranked from 0-100
    fn compare(monitor: &Monitor, db: &Db) -> () {
        let mut stmt = db.conn.prepare("
            SELECT uri, html, timestamp, sshash FROM pagesample WHERE uri = ?1 AND timestamp <= ?2
            ORDER BY timestamp DESC LIMIT 2").expect("Could not prepare query");
        let oldest_monitor = Utc::now().timestamp() - monitor.frequency;
        let page = stmt.query_row([&monitor.uri, &oldest_monitor.to_string()], |row| {
            Ok(Page {
                uri: row.get(0)?,
                html: row.get(1)?,
                sshash: row.get(3)?,
                timestamp: row.get(2)?
            })
        });

        let page2 = Page::new(&monitor.uri).expect("Could not create new page");
        let mut oldhash  = page.unwrap().sshash;
        let mut newhash = page2.sshash;
        let hashcomp = compare(&oldhash, &newhash);
        let mut similarity_check = Logging::new(page2.timestamp, monitor.uri.clone(), hashcomp.unwrap());
        Logging::check(similarity_check, &db, monitor.threshold);
        match hashcomp {
            Ok(hashcomp) => {
                println!("Hash comparison for: {}. URI Threshold recorded as: {}, hash similarity rating reported as: {}", monitor.uri, monitor.threshold, hashcomp );
            }
            Err(e) => {
                println!("Hash comparison for {} failed: {}", monitor.uri, e);
            }
        }
    }


    //create functionality that allows you to delete db objects of a certain age
    //modify this to utilize the URI for match case
    fn save(&mut self, db: &Db) {
        
        match &self.uri {
            Some(uri) => {
                let var = db.dbexecute(
                    "INSERT INTO pagesample (timestamp, uri, html, sshash) VALUES (?1, ?2, ?3, ?4)",
                    &[&self.timestamp, &self.uri, &self.html, &self.sshash ]
                );
            }
            None => {}

        }
    }
    //so far this is not going to function how I want it to.
    //option a: assign the page objects get all self.timestamp to variable called old_timestamp, and then pass this timestamp as a parameter to the remove function if old_timestamp <= frequency
    //option b: build this logic into the main function, which is not idiomatic in any way shape or form
    //option c: predetermine this in the monitor function(most likely best option)
    // as of current, the retention is statically set at 1 month. Witht that being said, there will be the importance of having this set on a per object basis
    fn remove(&mut self, db: &Db) {
        let mut data_retention = Local::now().timestamp() - 2629743;
        match &self.uri {
            Some(uri) => {
                let foo = db.dbexecute(
                    "DELETE FROM pagesample WHERE timestamp = ?1 AND uri = ?2",
                    &[&self.timestamp, &uri]
                );
            }
            None => {}
        }
    }
    // }
}


// to implement the logging features, i must have it so the comparison returns the page object
fn main() {
    let db = Db::new().unwrap();
    let mut destinations = Monitor::getall(&db).unwrap();
    let mut compare = destinations.iter().for_each(|m| Page::compare(&m, &db));

}





//    let mut destinations = Monitor::getall(&db).unwrap();
           
    
    
    
    
    
    
    
    
    
    
    
    // let uris = Monitor::repr(destinations);
    
//     for monitor in destinations {
//         Page::refresh(&monitor, &db);
//     }

    // let db = Db::new().unwrap();
    // // let mut destinations = Monitor::getall(&db).unwrap();
    // // let uris = Monitor::repr(destinations);
    //     let mut page1 = Page::new("").expect("Page object creation failed");
    //     Page::save( &mut page1 , &db);}

// let page1 = Page::new("https://www.google.com").unwrap().hash;
// let query = "SELECT * FROM pages WHERE type='monitored'";
// let execute = Db::new().map(|db| db.dbexecute(query, params));
// self.conn.execute(&statement, &params).unwrap();