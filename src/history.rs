use rusqlite::Connection;
use std::time::Duration;
use std::path::{Path,PathBuf};

// Note this database is locked when firefox is open so we'll need to make a copy of the file to access it
// could keep an md5 of the file to see if it changes between invocations and store it in tmp
// $HOME/.mozilla/firefox/ton7pjix.default-release/places.sqlite
// moz_places table

type FirefoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

#[derive(Debug)]
struct History {
    url: Option<String>,
    title: Option<String>,
    last_visit_date: Option<u32>, // it looks like this is stored in microseconds
}

pub struct Firefox {
    history_file: PathBuf,
}

impl Firefox {
    fn urls(&self, since: Option<Duration>) -> Result<std::vec::IntoIter<String>, FirefoxError> {
        let conn = Connection::open(self.history_file.as_path())?;
        Self::query_db(conn)
    }

    fn query_db(conn: Connection) -> Result<std::vec::IntoIter<String>, FirefoxError> {
        let mut stmt = conn.prepare("SELECT title, url, last_visit_date FROM moz_places ORDER BY last_visit_date DESC")?;
        let urls = stmt.query_map([], |row| {
            let history = History{
                url: row.get(1).ok(),
                title: row.get(0).ok(),
                last_visit_date: row.get(2).ok() as Option<u32>,
            };
            Ok(history)
        })?
            .filter_map(|history| history.unwrap().url)
            .collect::<Vec<String>>()
            .into_iter();

        Ok(urls)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Datelike, DateTime, NaiveDate, NaiveDateTime, Utc};

    #[test]
    fn test_read_from_local_firefox_db() {
        let path = Path::new("/home/erik/.mozilla/firefox/ton7pjix.default-release/places.sqlite").to_owned();
        let url_provider = Firefox{history_file: path};
        let urls = url_provider.urls(None);
        let url_string = urls.unwrap().next().unwrap();
        assert!(url_string.len() > 0);
    }

    #[test]
    fn test_time_from_nanos() {
        let dt = NaiveDateTime::from_timestamp(1_607_876_625, 287_119);
        assert_eq!(dt.year(), 2020);
        assert_eq!(dt.month(), 12);
    }

    #[test]
    fn test_query_db() -> Result<(), FirefoxError> {
        let db = Connection::open_in_memory()?;
        let sql = r#"
        CREATE TABLE moz_places (title LONGVARCHAR, url, LONGVARCHAR, last_visit_date INTEGER);
        "#;

        let last_visit_one: DateTime<Utc> = DateTime::from_utc(NaiveDate::from_ymd(2021, 11, 7).and_hms(0,0,0), Utc);
        let last_visit_two: DateTime<Utc> = DateTime::from_utc(NaiveDate::from_ymd(2021, 11, 1).and_hms(0,0,0), Utc);


        let insert_one = format!(r#"INSERT INTO moz_places(title, url, last_visit_date) VALUES ("title one", "http://url.one", {})"#, last_visit_one.timestamp() * 1000 * 1000);
        let insert_two = format!(r#"INSERT INTO moz_places(title, url, last_visit_date) VALUES ("title two", "http://url.two", {})"#, last_visit_two.timestamp() * 1000 * 1000);

        println!("insert_one: {}, insert_two: {}", insert_one, insert_two);
        Ok(())
    }
}
