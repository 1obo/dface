//Importing necessary crates and modules
mod args;
use chrono::*;
use image::*;
use image_hasher::*;
use rusqlite::*;
use ssdeep::*;
use std::fmt::{Debug};
use std::io::{BufRead, BufReader, Write, stdin, stdout};
use std::io::{Error, ErrorKind};
use std::string::String;
pub use rusqlite::fallible_iterator::FallibleIterator;
use rusqlite::{Connection, Result};
use thirtyfour::extensions::addons::firefox::FirefoxTools;
use thirtyfour::prelude::*;
//Object definition for the logging features
struct Logging {
    timestamp: u32,
    log_type: String,
    message: String,
}
//Definition for the Monitor object
#[derive(Debug)]
struct Monitor {
    uri: String,
    frequency: u32,
    threshold: u32,
    retention: u32,
}
// Structure for page object
#[derive(Debug)]
struct Page {
    uri: String,
    html: String,
    timestamp: u32,
    sshash: String,
    image: Option<Vec<u8>>,
    phash: String,
}

//Main function
fn main() -> Result<(), String> {
    //Parsing arguments passed in the command line
    let matches = args::get_command().get_matches();
    let filename = matches
        .get_one::<String>("output")
        .cloned()
        .expect("No output filename");
    let verbose = matches.get_flag("verbose");
    let new_monitor_uri = matches.get_one::<String>("New Monitor");
    let new_monitor_frequency = matches.
        get_one::<String>("New Monitor Frequency");
    let new_monitor_threshold = matches.get_one::<String>("New Monitor Threshold");
    let new_monitor_retention = matches.get_one::<String>("New Monitor Retention");
    let compare_old_page = matches.get_one::<String>("Compare Old Page URI");
    let print_similarities = matches.get_flag("Show hash similarities");
    let ignored_frequency = matches.get_flag("Ignore Frequency");
    let show_logs = matches.get_flag("Show logs");
    
    //Match case to unpack the comparison argument
    match compare_old_page {
        Some(compare_old_page) => println!(
            "Querying database for previous records pertaining to: {}",
            compare_old_page
        ),
        None => (),
    };
    //Match case to unpack the new monitor values
    match new_monitor_uri {
        Some(new_monitor_uri) => println!("new monitor uri: {} being added to DB", new_monitor_uri),
        None => (),
    };
    match new_monitor_frequency {
        Some(new_monitor_frequency) => {
            println!(
                "new monitor frequency: {} appended to new monitor",
                new_monitor_frequency
            )
        }
        None => (),
    }
    match new_monitor_retention {
        Some(new_monitor_retention) => {
            println!(
                "New monitor retention: {} appended to new monitor",
                new_monitor_retention
            )
        }
        None => (),
    }
    match new_monitor_threshold {
        Some(new_monitor_threshold) => {
            println!(
                "New monitor threshold: {} appended to new monitor",
                new_monitor_threshold
            )
        }
        None => (),
    }
    //Defining and creating the connection to database
    let conn = get_database_connection(
        new_monitor_uri,
        new_monitor_frequency,
        new_monitor_threshold,
        new_monitor_retention,
        &filename,
    )
    .expect("Failed to connect to database");

    //Printing logs if argument was passed to the program
    if show_logs==true {
        show_all_logs(&conn)
    }

    //Invoking the old page comparison function, if the argument was passed
    if compare_old_page.is_some() {
        let current_page = get_page(&compare_old_page.unwrap()).unwrap();

        let old_page = get_historical_page(compare_old_page, &conn).unwrap();

        let similarity = compare_pages(verbose, print_similarities, &current_page, &old_page);

        println!("Historical record has a fuzzy hash similarity of: {},\
        Perceptual Hash similarity of: {},\
        and an overall similarity of:{}", similarity[0], similarity[1], similarity[2]);



     //Main flow of the program
    } else {
        //Defining and creating the monitor objects
        let monitors = get_monitors(&conn);

        if verbose == true {
            println!("{:?}", monitors);
        }

        for monitor in monitors {
            //Deleting any pages aged beyond their monitors specified retention time
            delete_expired(&monitor.uri, &monitor.retention, &conn);
            //Defining and creating the most recent page objects for each monitor object
            let latest_page = get_latest_page(&monitor.uri, &conn);

            if latest_page.is_none() {
                if verbose == true {
                    println!(
                        "Current monitor:{} has no record in Page table, creating Page now...",
                        &monitor.uri
                    );
                    //Grabbing new page object with WebDriver
                    let page = get_page(&monitor.uri).expect("Failed to get page");
                    //Saving new page to database
                    save_page(&page, &conn).expect("Unable to save the page to database");

                } else if latest_page.is_some() {

                    let latest_page = latest_page.unwrap();
                    // Determining if the page frequency has been reached, ignored, or still within.
                    let cutoff_time = if ignored_frequency == true {
                        Utc::now().timestamp() as u32 - 0
                    } else {
                        Utc::now().timestamp() as u32 - &monitor.frequency
                    };
                    if latest_page.timestamp < cutoff_time {
                        if verbose == true {
                            println!("found an expired page, expired at: {}", cutoff_time);
                        }
                        let new_page = get_page(&monitor.uri).expect("Failed to get page");

                        save_page(&new_page, &conn).expect("Unable to save the page to database");
                        //Getting similarity value for the 2 distinct pages to be compared
                        let similarity =
                            compare_pages(verbose, print_similarities, &new_page, &latest_page);
                        //Checking similarity to determine alert, or log.
                        if similarity[0] < monitor.threshold {
                            let log_type: String = "ALERT".to_string();
                            //Building alert message
                            let message = format!(
                                "\
                        The uri:{:?} has been detected for potential defacement at timestamp:{:?}.\
                         Recorded cumulative hash similarity of: {:?}.\
                          SSHash similarity recorded as:{:?},\
                           and Phash similarity recorded as:{:?}",
                                &monitor.uri,
                                &new_page.timestamp,
                                &similarity[0],
                                &similarity[1],
                                &similarity[2]
                            );
                            //Building log and saving to database
                            let log = get_logs(&new_page.timestamp, &log_type, &message);
                            save_logs(&log, &conn).expect("Failed to save logs");
                            //Printing log message to terminal
                            println!("{}:{}", &log_type, &message)

                        } else {
                            let log_type: String = "LOG".to_string();
                            //Creating log message
                            let message = format!(
                                "\
                            The uri:{:?} has logged regular behaviour at timestamp:{:?}.\
                             Recorded cumulative hash similarity of: {:?}.\
                              SSHash similarity recorded as:{:?},\
                               and Phash similarity recorded as:{:?}",
                                &monitor.uri,
                                &new_page.timestamp,
                                &similarity[0],
                                &similarity[1],
                                &similarity[2]
                            );
                            let logs = get_logs(&new_page.timestamp, &log_type, &message);
                            save_logs(&logs, &conn).expect("Failed to save logs");
                            if verbose == true {
                                println!("{}:{}", &log_type, &message);
                            }
                        }
                    } else {
                        if verbose == true {
                            println!("found an unexpired page, expires at: {}", cutoff_time);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

//Function definition to query database, and return all log values to console
fn show_all_logs(conn: &Connection) -> () {
    let mut stmt = conn.prepare("\
    SELECT timestamp, log_type, message FROM logs"
    ).expect("Failed to prepare query");
    let rows = stmt.query_map([], |row| {
        let timestamp:String=row.get(0).expect("Failed to get Timestamp");
        let log_type:String=row.get(1).expect("Failed to get Log Type");
        let message:String=row.get(2).expect("Failed to get Log Message");

        Ok((timestamp, log_type, message))
    }).expect("Failed to execute query, please confirm the .sqlite file is present in working directory, and that the logs table exists within.");

    for result in rows {
        match result {
            Ok((timestamp, log_type, message)) => {
                println!("----------\nTimestamp: {}\nLog Type: {}\nMessage: {}", timestamp, log_type, message)
            }
            Err(e) => println!("Error in reading logs: {}", e),
        }
    }
    
}
//Function definition to compare new sample against a historical example
fn get_historical_page(compare_old_page: Option<&String>, conn: &Connection) -> Option<Page> {
    let mut stmt = conn.prepare(
            "SELECT rowid, uri, DATETIME(timestamp, 'localtime') as timestamp FROM pagesample WHERE uri = ?1 ORDER BY timestamp DESC"
        ).expect("Failed to prepare query");
    let mut rows = stmt.query(params![compare_old_page]).expect("Database Error: Please confirm .sqlite file is present under working directory, and that the provided URI is present in the Page Table");
    let output = rows.next().unwrap();
    let id: i64 = output.unwrap().get(0).unwrap();
    let historical_uri: String = output.unwrap().get(1).unwrap();
    let historical_timestamp: String = output.unwrap().get(2).unwrap();
    let prompt = format!(
        "----------\n Row ID: {} \n URI: {} \n Timestamp:{}\n Please type in the exact Row ID you would like to compare against current",
        id, historical_uri, historical_timestamp
    );

    let user_input = get_input(&prompt);

    let rowid = match user_input {
        Ok(clean_input) => {
            format!("{}", clean_input)
        }
        Err(e) => panic!("An error has occured sanitizing input: {}", e),
    };

    let mut stmt = conn.prepare(
            "SELECT uri, timestamp, html, sshash, phash FROM pagesample WHERE rowid = ?1 ORDER BY timestamp DESC LIMIT 1"
        )
            .expect(
                "Failed to prepare query"
            );
    let mut rows = stmt
        .query(params![&rowid])
        .expect("Failed to query results");
    let historical_page = if let Some(row) = rows.next().unwrap() {
        Some(Page {
            uri: row.get(0).expect("Failed to get page uri"),
            timestamp: row.get(1).expect("Failed to get page timestamp"),
            html: row.get(2).expect("Failed to get page html"),
            image: None,
            sshash: row.get(3).expect("Failed to get page sshash"),
            phash: row.get(4).expect("Failed to get page phash"),
        })
    } else {
        None
    };
    historical_page
}
//Function definition to parse command line input for historical page comparison
fn get_input(prompt: &str) -> Result<String, Error> {
    print!("{}", prompt);
    stdout().flush().expect("Argument error: Unable to flush stdout");
    let input = BufReader::new(stdin())
        .lines()
        .next()
        .ok_or_else(|| Error::new(ErrorKind::Other, "Cannot read stdin"))
        .and_then(|inner| inner);
    input
}
//Function definition to ouput log data to console
fn get_logs(
    page_timestamp: &u32,
    log_type: &String,
    message: &String
) -> Logging {
    Logging {
        timestamp: page_timestamp.to_owned(),
        log_type: log_type.to_string(),
        message: message.to_string(),
    }
}
//Function definition to save log data to SQLite database
fn save_logs(log: &Logging, conn: &Connection) -> Result<usize> {
    conn.execute(
        "INSERT INTO logs
                    (timestamp, log_type, message)
                    VALUES (?1, ?2, ?3)",
        params![log.timestamp, log.log_type, log.message],
    )
}
//Function definition for page comparison
fn compare_pages(verbose: bool, show_similarities: bool, page1: &Page, page2: &Page) -> [u32; 3] {
    println!("1 {} and 2 {}", &page1.phash, &page2.phash);
    let old_sshash = &page1.sshash;
    let new_sshash = &page2.sshash;
    let old_phash: ImageHash<Box<[u8]>> =
        ImageHash::from_base64(&page1.phash).expect("Failed to get ImageHash");
    let new_phash: ImageHash<Box<[u8]>> =
        ImageHash::from_base64(&page2.phash).expect("Failed to get ImageHash");
    let sshashcomp = compare(&old_sshash, &new_sshash).expect("Failed to compare pages");
    let phash_ham_dist = &old_phash.dist(&new_phash);
    let phashcomp: u32 = 100 - ((100 * phash_ham_dist) / 72);
    let similarity_rating = (sshashcomp + phashcomp) / 2;
    if verbose == true {
        println!(
            "sshash similarity: {:?}, phashcomp similarity: {:?}, overall similarity: {:?}",
            sshashcomp, phashcomp, similarity_rating
        );
    } else if show_similarities == true {
        println!(
            "sshash similarity: {:?}, phashcomp similarity: {:?}, overall similarity: {:?}",
            sshashcomp, phashcomp, similarity_rating
        );
    }
    let diff = [similarity_rating, sshashcomp, phashcomp];
    diff
}
//Function definition to save pages to database
fn save_page(page: &Page, conn: &Connection) -> Result<usize> {
    conn.execute(
        "INSERT INTO pagesample \
                (uri, timestamp, html, image, sshash, phash) \
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            &page.uri,
            &page.timestamp,
            &page.html,
            &page.image,
            &page.sshash,
            &page.phash
        ],
    )
}
//Function definition to create new page objects using geckodriver
#[tokio::main]
async fn get_page(uri: &String) -> Option<Page> {
    let mut capabilities = DesiredCapabilities::firefox();
    capabilities.add_arg("--headless").expect("Firefox driver not accessible to WebDriver, please ensure you have gecko driver installed");

    let driver = WebDriver::new("http://localhost:4444", capabilities)
        .await
        .expect("Failed to connect to WebDriver");
    let tools = FirefoxTools::new(driver.handle.clone());
    driver
        .goto(uri)
        .await
        .expect("Failed to get URI from WebDriver");
    tokio::time::sleep(Duration::seconds(3).to_std().unwrap()).await;
    let html = driver
        .source()
        .await
        .expect("Failed to get HTML from WebDriver");
    let screenshot = tools.full_screenshot_as_png().await.unwrap();
    let timestamp = Utc::now().timestamp();
    driver.quit().await.expect("Failed to quit WebDriver");
    let sshash = get_sshash(&html);
    let phash = get_phash(&screenshot);
    Some(Page {
        uri: uri.to_string(),
        timestamp: timestamp as u32,
        html,
        image: Some(screenshot),
        sshash,
        phash,
    })
}
//Function definition to grab phash of new page screenshot
fn get_phash(image: &Vec<u8>) -> String {
    let input = load_from_memory(image).expect("Failed to load image");
    let hasher = HasherConfig::new().to_hasher();
    let phash = hasher.hash_image(&input);
    phash.to_base64()
}
//Function definition to grab fuzzy hash of new page source code
fn get_sshash(html: &String) -> String {
    let sshash = hash_buf(&html.as_bytes());
    sshash.unwrap().to_string()
}
//Function definition to get most recent page object from database
fn get_latest_page(uri: &String, conn: &Connection) -> Option<Page> {
    let mut stmt = conn
            .prepare("SELECT uri, timestamp, html, sshash, phash FROM pagesample WHERE uri = ?1 ORDER BY timestamp DESC LIMIT 1").expect("Failed to prepare statement");
    let mut rows = stmt.query(params![&uri]).expect("Failed to query results");
    let latest_page = if let Some(row) = rows.next().unwrap() {
        Some(Page {
            uri: row.get(0).expect("Failed to get page uri"),
            timestamp: row.get(1).expect("Failed to get page timestamp"),
            html: row.get(2).expect("Failed to get page html"),
            image: None,
            sshash: row.get(3).expect("Failed to get page sshash"),
            phash: row.get(4).expect("Failed to get page phash"),
        })
    } else {
        None
    };
    latest_page
}
//Function definition to delete expired page objects
fn delete_expired(uri: &String, retention: &u32, conn: &Connection) {
    let pageretention = Utc::now().timestamp() as u32 - retention;
    conn.execute(
        "\
    DELETE FROM pagesample WHERE uri = ?1 AND timestamp < ?2",
        params![uri, pageretention],
    )
    .expect("Failed to delete expired page");
}
//Function definition to get all monitor objects from database
fn get_monitors(conn: &Connection) -> Vec<Monitor> {
    let mut stmt = conn
        .prepare("SELECT  uri, frequency, threshold, retention FROM monitored")
        .expect("Failed to prepare statement");
    //building query map to correlate incoming data to monitor fields
    let monitors = stmt
        .query_map([], |row| {
            Ok(Monitor {
                uri: row.get(0)?,
                frequency: row.get(1)?,
                threshold: row.get(2)?,
                retention: row.get(3)?,
            })
        })
        .expect("Failed to get monitors")
        .collect::<Result<Vec<Monitor>>>()
        .expect("Failed to output results");
    monitors
}
//Function definition to build database connection
fn get_database_connection(
    new_monitor_uri: Option<&String>,
    new_monitor_frequency: Option<&String>,
    new_monitor_threshold: Option<&String>,
    new_monitor_retention: Option<&String>,
    file: &String,
) -> Result<Connection> {
    //Creating all necessary tables in database
    let conn = Connection::open(file)?;
    conn.execute(
        "
                CREATE TABLE IF NOT EXISTS monitored (
	            uri TEXT PRIMARY KEY,
	            frequency INTEGER NOT NULL,
	            threshold INTEGER NOT NULL,
                retention INTEGER NOT NULL
                )",
        [],
    ).expect("Failed to create monitor table");
    conn.execute(
        "
                CREATE TABLE IF NOT EXISTS pagesample (
                uri TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                html TEXT NOT NULL,
                image BLOB NOT NULL,
                sshash TEXT NOT NULL,
                phash TEXT NOT NULL,
                PRIMARY KEY(uri,timestamp)
                )",
        [],
    ).expect("Failed to create page table");
    conn.execute(
        "
                CREATE TABLE IF NOT EXISTS logs (
                timestamp INTEGER NOT NULL,
                log_type TEXT NOT NULL,
                message TEXT NOT NULL
                )",
        [],
    ).expect("Failed to create logs table");
    //Creating new monitor object if argument was passed
    if new_monitor_uri.is_some() {
        if new_monitor_frequency.is_some() {
            let frequency:u32 = new_monitor_frequency.unwrap().parse().expect("Failed to parse frequency string");
            if new_monitor_threshold.is_some() {
                let threshold:u32 = new_monitor_threshold.unwrap().parse().expect("Failed to parse threshold string");
                if new_monitor_retention.is_some() {
                    let retention:u32 = new_monitor_retention.unwrap().parse().expect("Failed to parse retention string");
                    conn.execute(
                        "\
                 INSERT INTO monitored\
                 (uri, frequency, threshold, retention) \
                 VALUES (?1, ?2, ?3, ?4)\
                 ",
                        params![
                            new_monitor_uri,
                            frequency,
                            threshold,
                            retention
                        ],
                    ).expect("Failed to add new monitor to table");
                } else {
                    panic!(
                        "No monitor retention provided, please ensure you're using all parameters associated with monitor creation,and are passing the types specified in --help"
                    );
                }
            } else {
                panic!(
                    "No monitor threshold provided, please ensure you're using all parameters associated with monitor creation,and are passing the types specified in --help"
                )
            }
        } else {
            panic!(
                "No monitor frequency provided, please ensure you're using all parameters associated with monitor creation,and are passing the types specified in --help"
            )
        }
    }

    Ok(conn)
}
