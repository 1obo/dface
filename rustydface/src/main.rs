#![allow(unused)]
use chrono::*;
use futures::future::join_all;
use image::*;
use image_hasher::*;
use rusqlite::fallible_iterator::FallibleIterator;
use rusqlite::*;
use ssdeep::*;
use std::error::Error;
use std::fmt::{Debug, Display};
use std::string::String;
use std::process::Command;
use std::sync::BarrierWaitResult;
use thirtyfour::extensions::addons::firefox::FirefoxTools;
use thirtyfour::FirefoxCapabilities;
use thirtyfour::prelude::*;
use tokio::spawn;
use tokio::task;
struct Logging {
    timestamp: u32,
    log_type: String,
    message: String,
}
#[derive(Debug)]
#[derive(Clone)]
struct Monitor {
    uri: String,
    frequency: u32,
    threshold: u32,
    retention: u32,
}
#[derive(Debug)]
// Structure for page object
struct Page {
    uri: String,
    html: String,
    timestamp: u32,
    sshash: String,
    image: Option<Vec<u8>>,
    phash: String,
}
#[tokio::main]
async fn main() -> Result<(), String> {
    // Creating the database connection
    let filename = String::from("dface.sqlite");
    let conn = get_data_base_connection(&filename).expect("Failed to get database connection");

    //Get all monitors
    let monitors = get_monitors(&conn);
    println!("{:?}", monitors);
    //if monitor.retention > monitor.frequency:
    for monitor in &monitors {
        if &monitor.frequency >= &monitor.retention {
            return Err(format!(
                "Configuration error: The frequency: {:?} for {:?} has exceeded the retention value of {:?}.",
                &monitor.frequency, &monitor.uri, &monitor.retention
            ));
        }
    }
    let mut driverlist: Vec<&WebDriver> = vec![];
    // let mut foo:u32 = 0;
    // let mut var:u32 = 4444;
    for monitor in &monitors {
        // foo +=1;
        // let port = foo+var;
        // port.to_string();
        // let command = format!(".\\geckodriver.exe --port={:?}", port);
        // let cmd = Command::new(command);
        let mut capabilities = DesiredCapabilities::firefox();
        capabilities.add_arg("--headless");
        let driver = WebDriver::new("http://localhost:4444", capabilities)
            .await
            .expect("Failed to start webdriver");
        driverlist.push(&driver);
        delete_expired(&monitor.uri, &monitor.retention, &conn);
        let latest_page = get_latest_page(&monitor.uri, &conn);
        if latest_page.is_none() {
            let jobs: Vec<_> = monitors
                .clone()
                .into_iter()
                .map(|monitor| {
                    let mut uri = monitor.uri.clone();
                    println!("No page found for {:?}. Creating page now...", &monitor.uri);
                    let handle = spawn(get_page(uri, &driver, &capabilities));
                    handle
                })
                .collect();
            let results = join_all(jobs).await;


            for result in results {
                match result {
                    Ok(Some(page)) => {
                        if let Err(e) = save_page(&page, &conn) {
                            println!("Error saving page: {}", e);
                        }
                    }
                    Ok(None) => println!("No page found for {:?}!", &monitor.uri),
                    Err(e) => println!("Error: {:?}", e),
                }
            }
        } else {
            let latest_page = latest_page.unwrap();
            let cutoff_time = (Utc::now().timestamp() as u32 - monitor.frequency);
            if latest_page.timestamp < cutoff_time {
                println!("found an expired page, expired at: {}", cutoff_time);
                //create a page for that monitor
                let jobs: Vec<_> = monitors
                    .clone()
                    .into_iter()
                    .map(|monitor| {
                        let mut uri = monitor.uri.clone();
                        println!("No page found for {:?}. Creating page now...", &monitor.uri);
                        let handle = spawn(get_page(uri, &driver, &capabilities));
                        handle
                    })
                    .collect();
                let results = join_all(jobs).await;
                for result in results {
                    match result {
                        Ok(Some(page)) => {
                            //saving pages, and println! if error occurs without disrupting.
                            if let Err(e) = save_page(&page, &conn) {
                                println!("Error saving page: {}", e);
                                let diff = compare_pages(&page, &latest_page);
                                if diff < monitor.threshold {
                                    let log_type: String = "ALERT".to_string();
                                    let message = format!(
                                        "\
                                                The uri:{:?} has been detected for potential defacement at timestamp:{:?}. Recorded cumulative hash similarity of: {:?}",
                                        &monitor.uri, &page.timestamp, &diff
                                    );
                                    //create alert/log
                                    //store alert/log to database
                                    let log = get_logs(&page.timestamp, &log_type, &message);
                                    save_logs(&log, &conn).expect("Failed to save logs");
                                    println!("{}:{}", &log_type, &message);
                                } else {
                                    let log_type: String = "LOG".to_string();
                                    let message = format!(
                                        "\
                                            The uri:{:?} has logged regular behaviour at timestamp:{:?}. Recorded cumulative hash similarity of: {:?}",
                                        &monitor.uri, &latest_page.timestamp, &diff
                                    );
                                    let logs = get_logs(&page.timestamp, &log_type, &message);
                                    save_logs(&logs, &conn).expect("Failed to save logs");
                                    println!("{}:{}", &log_type, &message);
                                }
                            }
                        }

                        Ok(None) => println!("No page found for {:?}!", &monitor.uri),
                        Err(e) => println!("Error: {:?}", e),
                    }
                }
                // driver.quit()
                // .await
                // .expect("Failed to quit webdriver");
            } else {
                println!("found an unexpired page, expires at: {}", cutoff_time);
            }
        }
    }

    for driver in driverlist.into_iter() {
        &driver.quit();
    }

    Ok(())
}
fn get_logs(page_timestamp: &u32, log_type: &String, message: &String) -> Logging {
    Logging {
        timestamp: page_timestamp.to_owned(),
        log_type: log_type.to_string(),
        message: message.to_string(),
    }
}
fn save_logs(log: &Logging, conn: &Connection) -> Result<usize> {
    conn.execute(
        "INSERT INTO logs
                    (timestamp, log_type, message)
                    VALUES (?1, ?2, ?3)",
        params![log.timestamp, log.log_type, log.message],
    )
}
fn compare_pages(page1: &Page, page2: &Page) -> u32 {
    println!("1 {} and 2 {}", &page1.phash, &page2.phash);
    let old_sshash = &page1.sshash;
    let new_sshash = &page2.sshash;
    let old_phash: ImageHash<Box<[u8]>> =
        ImageHash::from_base64(&page1.phash).expect("Failed to get ImageHash");
    let new_phash: ImageHash<Box<[u8]>> =
        ImageHash::from_base64(&page2.phash).expect("Failed to get ImageHash");
    let sshashcomp = compare(&old_sshash, &new_sshash).expect("Failed to compare pages");
    let phash_ham_dist = &old_phash.dist(&new_phash);
    let phashcomp: u32 = 100 * (1 - phash_ham_dist / 72);
    let similarity_rating = (sshashcomp + phashcomp) / 2;
    println!(
        "sshash similarity: {:?}, phashcomp similarity: {:?}, overall similarity: {:?}",
        sshashcomp, phashcomp, similarity_rating
    );
    similarity_rating
}
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
async fn get_page(uri: String, driver: &WebDriver, capabilities: &FirefoxCapabilities) -> Option<Page> {
    // let mut capabilities = DesiredCapabilities::firefox();
    // capabilities.add_arg("--headless");
    // let driver_port = format!("http://localhost:4444", capabilities);
    // let driver = WebDriver::new(driver_port, capabilities)
    //     .await
    //     .expect("Failed to connect to WebDriver");
    let tools = FirefoxTools::new(driver.handle.clone());
    driver
        .goto(&uri)
        .await
        .expect("Failed to get URI from WebDriver");
    tokio::time::sleep(Duration::seconds(3).to_std().unwrap());
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
fn get_phash(image: &Vec<u8>) -> String {
    let input = load_from_memory(image).expect("Failed to load image");
    let hasher = HasherConfig::new().to_hasher();
    let phash = hasher.hash_image(&input);
    phash.to_base64()
}
fn get_sshash(html: &String) -> String {
    let sshash = hash_buf(&html.as_bytes());
    sshash.unwrap().to_string()
}
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
fn delete_expired(uri: &String, retention: &u32, conn: &Connection) {
    let pageretention = Utc::now().timestamp() as u32 - retention;
    conn.execute(
        "\
    DELETE FROM pagesample WHERE uri = ?1 AND timestamp < ?2",
        params![uri, pageretention],
    )
    .expect("Failed to delete expired page");
}
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
fn get_data_base_connection(file: &String) -> Result<Connection> {
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
    );
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
    );
    conn.execute(
        "
                CREATE TABLE IF NOT EXISTS logs (
                timestamp INTEGER NOT NULL,
                log_type TEXT NOT NULL,
                message TEXT NOT NULL
                )",
        [],
    );
    conn.execute(
        "\
                 INSERT INTO monitored\
                 (uri, frequency, threshold, retention) \
                 VALUES (?1, ?2, ?3, ?4)\
                 ",
        params![String::from("https://www2.gov.bc.ca/gov/content/home"), 600, 80, 86400],
    );
    Ok(conn)
}
