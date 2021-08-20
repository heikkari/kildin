// rusqlite
use rusqlite::Connection;
use rusqlite::params;

// crate
use crate::helpers::types;
use crate::database::proxies::Proxy;
use crate::connect_to_database;

type Entries = Vec<RateLimitEntry>;

pub struct RateLimited {
    conn: Connection
}

#[derive(Clone)]
pub struct RateLimitEntry {
    pub website: String,
    pub address: String,
    pub port: u16,
    pub until: u64
}

impl RateLimited {
    pub fn new() -> Result<Self, types::AnyError> {
        let conn = connect_to_database()?;
        Ok(Self { conn })
    }

    pub fn create(conn: Connection) {
        conn.execute(
            "
                CREATE TABLE IF NOT EXISTS ratelimited (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    website TEXT,
                    address TEXT,
                    port INTEGER,
                    until INTEGER,
                    UNIQUE (website, address, port)
                )
            ",
            rusqlite::NO_PARAMS
        ).unwrap();
    }

    pub fn add(&mut self, entries: Entries) -> Result<(), types::AnyError> {
        let trs = self.conn.transaction()?;

        for e in entries.iter() {
            let query = "INSERT OR IGNORE INTO ratelimited (website, address, port, until) VALUES (?1, ?2, ?3, ?4)";
            trs.execute(query, params![e.website, e.address, e.port, e.until as i64])?;
        }

        trs.commit()?;
        Ok(())
    }

    pub fn remove(&mut self, entries: Entries) -> Result<(), types::AnyError> {
        let trs = self.conn.transaction()?;

        for e in entries.iter() {
            let query = "DELETE FROM ratelimited WHERE website = ?1 AND address = ?2 AND port = ?3";
            trs.execute(query, params![e.website, e.address, e.port])?;
        }

        trs.commit()?;
        Ok(())
    }

    pub fn get_ratelimited(&mut self, website: &str, proxies: Vec<Proxy>)
        -> Result<Vec<Proxy>, types::AnyError>
    {
        let trs = self.conn.transaction()?;
        let mut ratelimited = Vec::new();

        for p in proxies {
            let query = "SELECT * FROM ratelimited WHERE (website = ?1 OR website = '*') AND address = ?2 AND port = ?3";

            // query
            let params = params![website, p.address, p.port];
            let mut stmt = trs.prepare(query)?;
            let rows: _ = stmt.query_map(params, |_| Ok(1));

            if rows.is_err() {
                continue;
            }

            if rows.unwrap().size_hint().0 != 0 {
                ratelimited.push(p);
            }
        }

        Ok(ratelimited)
    }

    fn after(&self, from: u32, pag: u32) -> Result<Vec<(u32, RateLimitEntry)>, types::AnyError> {
        // set up query
        let query = "SELECT * FROM ratelimited WHERE id > ?1 ORDER BY id ASC LIMIT ?2";
        let params = params![from, pag];

        // execute query
        let mut vec = Vec::new();
        let mut rows = self.conn.prepare(query)?;
        let rows: _ = rows.query_map(params, |row| {
            // get row info
            let key: u32 = row.get(0)?;
            let website: String = row.get(1)?;
            let address: String = row.get(2)?;
            let port: u16 = row.get(3)?;
            let until: i64 = row.get(4)?;

            Ok((key, RateLimitEntry { website, address, port, until: until as u64 }))
        })?;

        // collect rows
        for row in rows {
            vec.push(row?);
        }

        Ok(vec)
    }

    pub fn read_ratelimited<F, W>(&self, pag: u32, f: F)
        -> Result<Vec<W>, types::AnyError>
        where F: Fn(Vec<(u32, RateLimitEntry)>) -> Vec<W>
    {
        let mut idx = 0;
        let mut proxies = Vec::new();

        loop {
            // get vec and set idx
            let vec = self.after(idx, pag)?;

            if vec.len() == 0 {
                break;
            }

            idx = vec[vec.len() - 1].0;
            proxies.append(&mut f(vec));
        }

        Ok(proxies)
    }
}