#![allow(unused)]
use chrono::*;
use rusqlite::*;
use ssdeep::*;
use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Display};
use std::string::String;
use thirtyfour::prelude::*;


struct Logging {
    timestamp: i64,
    uri: String,
    similarity: u32,
}
//Simple logging implementation for SQLite log table
impl Logging {
    //Constructs new logging object
    fn new(timestamp: i64, uri: String, similarity: u32) -> Logging {
        Logging {timestamp, uri, similarity }
    }
    
    //Checking hash similarity against allowed threshold
    fn check(logs:Logging, db: &Db, threshold:u32) {
        if logs.similarity < threshold {
            //Executing query accordingly
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
      }

impl Monitor {
    //Constructing new Monitor object
    pub fn new(uri: &str, frequency: i64, threshold: u32) -> Monitor {
        Monitor { id:None, uri:uri.to_string(), frequency, threshold }
    }

    //Constructing Monitor object in case that we are simply updating info already correlated to ID
    fn newwithid(id: i64, uri: &str, frequency: i64, threshold:u32) -> Monitor {
        Monitor { id:Some(id), uri:uri.to_string(), frequency, threshold }
    }
    // Obtaining critical page info from Monitor table
    fn getall(db: &Db) -> Result<Vec<Monitor>> {
        //Querying DB to populate monitor fields
        let mut stmt = db.conn.prepare(
            "SELECT id, uri, frequency, threshold FROM monitored"
        )?;
        //building query map to correlate incoming data to monitor fields
        let destinations = stmt.query_map([], |row| {
            Ok(Monitor{
                id: Some(row.get(0)?),
                uri:row.get(1)?,
                frequency:row.get(2)?,
                threshold:row.get(3)?
            })
        })?
            //collecting result set to return the monitor object
            .collect::<Result<Vec<Monitor>>>()?;
            Ok(destinations)
    }
    //representation of the URI's into a string vector for iteration in main()
    fn repr(destinations: Vec<Monitor>) -> Vec<String> {
        destinations.into_iter().map(|m| m.uri).collect()

    }
    //function to save new objects in monitor table, or update existing objects
    fn save(&mut self, db: &Db) {
        match &self.id {
            Some(id) => {
                //Updating existing monitor object
                let var = db.dbexecute(
                    "UPDATE monitored SET uri = ?1, frequency = ?2 WHERE id = ?3",
                &[&self.uri, &self.frequency, id]);

            }
            None => {
                //Inserting new monitor object into table
                let foo = db.dbexecute("INSERT INTO monitored (uri, frequency) VALUES (?1, ?2)",
                    &[&self.uri, &self.frequency]);
                let id = db.conn.last_insert_rowid();
                self.id = Some(id);
            }
        }
    }



}
// Simple display implementation for monitor objects
impl Display for Monitor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "id:{:?}, uri:{:?}, frequency:{:?}", self.id, self.uri, self.frequency)
    }
}

//structure for Database object
struct Db {
    file: String,
    conn: Connection,
}


impl Db {
    //Creation of database object
    fn new() -> Result<Db, Box<dyn Error>> {
        //Specified filename statically coded in this case
        let file = "dface.sqlite".to_string();
        //Connection definition for reference in main()
        let conn = Connection::open(file.clone()).expect("sqlite open error");
        Ok(Db { file, conn })
    }
    // Function to execute SQL in SQLite DB
    fn dbexecute(&self, statement: &str, params: &[&dyn ToSql]) -> Result<()> {
        self.conn.execute(statement, params).expect("dbexecute error");
        Ok(())
        
    }
}


#[derive(Debug)]
// Structure for page object
struct Page {
    uri: Option<String>,
    html: String,
    timestamp: i64,
    sshash: String,

}
impl Page {
    #[tokio::main]
    //Asynchronus function for creation of new page object
    async fn new(uri: &str) -> WebDriverResult<Self> {
        //Specifying webdriver capabilities
        let capabilities = DesiredCapabilities::chrome();
        //Creating webdriver object
        let driver = WebDriver::new("http://localhost:11111", capabilities).await.expect("Failed to connect to WebDriver");
        //Setting browser resolution for future image hashing capabilities
        driver.set_window_rect(1, 1, 1920, 1080);
        //Directing browser to uri sourced from the page structure
        driver.goto(uri).await.expect("Failed to get URI from WebDriver");
        //grabbing html from specified webpage
        let html = driver.source().await.expect("Failed to get HTML from WebDriver");
        //grabbing byte array of png screenshot for perceptual hashing
        let image = driver.screenshot_as_png().await.expect("Failed to get image from WebDriver");
        //definiing timestamp as current time in UTC
        let timestamp = Utc::now().timestamp();
        //Closing browser window
        driver.quit().await;
        //Hashing html
        let sshash = Self::hashvalues(&html).await;
        //Instantiating page object as a result set
        Ok(
            Page
            {
                uri: Some(uri.to_string()),
                html,
                timestamp,
                sshash,
            })
    }

    //Using ffuzzy/ssdeep to generate ffuzzy hashes of html
    async fn hashvalues(html: &str) -> String {
        let sshash1 = hash_buf(html.as_bytes());
        sshash1.unwrap().to_string()
    }
    //function to obtain all page objects from page table
    fn getall(db: &Db) -> Result<Vec<Page>> {
        //querying DB
        let mut stmt = db.conn.prepare(
            "SELECT uri, html, timestamp, sshash FROM pagesample"
        )?;
        //Building querymap for retrieved data correlation to page values
        let sources = stmt.query_map([], |row| {
            Ok(Page{
                uri: Some(row.get(0)?),
                html: row.get(1)?,
                sshash: row.get(3)?,
                timestamp: row.get(6)?
            })
        })?
            //Collecting for use within main()
        .collect::<Result<Vec<Page>>>()?;
        Ok(sources)

    }
    //Function that compares last recorded hash value for html from DB to freshly acquired value, as well as update tables
    fn compare(monitor: &Monitor, db: &Db) -> () {
        //preparing query for execution
        let mut stmt = db.conn.prepare("
            SELECT uri, html, timestamp, sshash FROM pagesample WHERE uri = ?1 AND timestamp <= ?2
            ORDER BY timestamp DESC LIMIT 2").expect("Could not prepare query");
        //Defining frequency value that specifies an interval in which these values are considered expired
        let oldest_monitor = Utc::now().timestamp() - monitor.frequency;
        //Executing query, and returning values into rows for correlation to Page values
        let page = stmt.query_row([&monitor.uri, &oldest_monitor.to_string()], |row| {
            Ok(Page {
                uri: row.get(0)?,
                html: row.get(1)?,
                sshash: row.get(3)?,
                timestamp: row.get(2)?
            })
        });
        //Creating new instance of page object using URI
        let page2 = Page::new(&monitor.uri).expect("Could not create new page");
        //Creating variable that holds previous fuzzy hash
        let mut oldhash  = page.unwrap().sshash;
        //Creating variable that holds recently obtained fuzzy hash
        let mut newhash = page2.sshash;
        //Comparing hashes, outputting a similarity score from 0-100
        let hashcomp = compare(&oldhash, &newhash);
        //Instantiating Logging object
        let mut similarity_check = Logging::new(page2.timestamp, monitor.uri.clone(), hashcomp.unwrap());
        //Checking similarity score against monitored URI threshold and alerting if necessary
        Logging::check(similarity_check, &db, monitor.threshold);
        //Match case for error handling
        match hashcomp {
            Ok(hashcomp) => {
                println!("Hash comparison for: {}. URI Threshold recorded as: {}, hash similarity rating reported as: {}", monitor.uri, monitor.threshold, hashcomp );
            }
            Err(e) => {
                println!("Hash comparison for {} failed: {}", monitor.uri, e);
            }
        }
    }

    //Save function to generate new pages in table
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
    //Remove function to delete rows beyond specified retention(Statically coded @ 1mo/enoch
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
}


fn main() {
    let db = Db::new().unwrap();
    let mut destinations = Monitor::getall(&db).unwrap();
    let mut compare = destinations.iter().for_each(|m| Page::compare(&m, &db));

}
