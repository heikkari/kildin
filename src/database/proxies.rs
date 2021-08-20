// - proxies [index, schema, proxy address, rating]

// rusqlite
use rusqlite::Connection;
use rusqlite::params;

// crate
use crate::helpers::types;
use crate::connect_to_database;

// serde
use serde_derive::Serialize;

macro_rules! query_proxies {
    ($stmt:expr, $params:expr) => {
        $stmt.query_map($params, |row| {
            // get row info
            let key: u32 = row.get(0)?;
            let schema: String = row.get(1)?;
            let address: String = row.get(2)?;
            let port: u16 = row.get(3)?;
            let rating: f64 = row.get(4)?;
            let fails: u32 = row.get(5)?;
            let blacklisted: bool = row.get(6)?;

            Ok((key, Proxy { schema, address, port,
                rating, fails, blacklisted }))
        })
    };
}

macro_rules! order_proxies {
    ($name:ident, $param:ident, $param_type:ident, $column:expr) => {
        pub fn $name(&self, $param: $param_type, pag: u32) -> Result<Vec<(u32, Proxy)>, types::AnyError> {
            // set up query
            let query = format!("SELECT * FROM proxies WHERE {} > ?1 AND blacklisted = 0 ORDER BY {} ASC LIMIT ?2", stringify!($column), stringify!($column));
            let params = params![$param, pag];

            // execute query
            let mut vec = Vec::new();
            let mut stmt = self.conn.prepare(&query)?;

            // map over rows
            let rows: _ = query_proxies!(stmt, params)?;

            // collect rows
            for row in rows {
                vec.push(row?);
            }

            Ok(vec)
        }
    };
}

macro_rules! bulk_sql {
    ($conn:expr, $proxies:expr, $query:expr, $row_name:ident, $params:expr) => {
        let trs = $conn.transaction()?;

        for $row_name in $proxies.iter() {
            trs.execute($query, $params)?;
        }

        trs.commit()?;
    };
}

macro_rules! bulk_sql_function {
    ($f:ident, $query:expr, $row_name:ident, $params:expr) => {
        pub fn $f(&mut self, proxies: Vec<Proxy>)
            -> Result<(), types::AnyError>
        {
            bulk_sql!(self.conn, proxies, $query, $row_name, $params);
            Ok(())
        }
    };
}

#[derive(Clone, Serialize, PartialEq)]
pub struct Proxy {
    pub schema: String,
    pub address: String,
    pub port: u16,
    pub rating: f64,
    pub fails: u32,
    pub blacklisted: bool,
}

pub struct Proxies {
    conn: Connection,
}

impl Proxies {
    pub fn new() -> Result<Self, types::AnyError> {
        let conn = connect_to_database()?;
        Ok(Self { conn })
    }

    // TODO: Store IP address as integer instead of text
    pub fn create(conn: Connection) {
        conn.execute(
            "
                CREATE TABLE IF NOT EXISTS proxies (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    schema_ TEXT,
                    address TEXT,
                    port NUMBER,
                    rating REAL,
                    fails NUMBER,
                    blacklisted NUMBER,
                    UNIQUE (address, port)
                )
            ",
            rusqlite::NO_PARAMS
        ).unwrap();
    }

    pub fn top_rated(&self, limit: u32) -> Result<Vec<(u32, Proxy)>, types::AnyError> {
        let mut proxies = Vec::new();

        // query proxies
        let query = "SELECT * FROM proxies WHERE blacklisted = 0 ORDER BY rating DESC LIMIT ?1";
        let mut stmt = self.conn.prepare(query)?;
        let rows = query_proxies!(stmt, params![limit])?;

        for row in rows {
            proxies.push(row?);
        }

        //proxies.sort_by(|a, b| b.1.rating.partial_cmp(&a.1.rating).unwrap());
        Ok(proxies)
    }

    // bulk sql functions
    bulk_sql_function!(insert_proxies,
        "INSERT OR IGNORE INTO proxies (schema_, address, port, rating, fails, blacklisted) VALUES (?1, ?2, ?3, ?4, ?5, ?6) WHERE blacklisted = 0",
        proxy, params![proxy.schema, proxy.address, proxy.port, proxy.rating, proxy.fails, proxy.blacklisted]);

    bulk_sql_function!(update_proxies,
        "UPDATE proxies SET rating = ?1, fails = ?2, blacklisted = ?3 WHERE (address = ?4 AND port = ?5) AND blacklisted = 0",
        proxy, params![proxy.rating, proxy.fails, proxy.blacklisted, proxy.address, proxy.port]);

    bulk_sql_function!(delete_proxies,
        "DELETE FROM proxies WHERE address = ?1 AND port = ?2 AND blacklisted = 0",
        proxy, params![proxy.address, proxy.port]);

    // order functions
    order_proxies!(after, from, u32, "id");
    order_proxies!(over, min_rating, f64, "rating");
}