// token format:
// alphanumeric 32 characters

// rusqlite
use rusqlite::types::Value;
use rusqlite::Connection;
use rusqlite::params;

// std
use std::cmp::PartialEq;

// crate
use crate::helpers::types;
use crate::connect_to_database;

pub enum ManagerResult<T> {
    Ok(T),
    Err(&'static str)
}

#[derive(PartialEq)]
pub enum ManagerState {
    Disabled, // 0
    Ok, // 1
    Admin, // 2
    Unknown
}

impl Into<ManagerState> for u8 {
    fn into(self) -> ManagerState {
        match self {
            0 => ManagerState::Disabled,
            1 => ManagerState::Ok,
            2 => ManagerState::Admin,
            _ => ManagerState::Unknown
        }
    }
}

pub struct TableInfo {
    pub table_name: String,
    pub token_row_name: String,
    pub manager_state_row_name: String,
}

pub struct ManagerAuth {
    conn: Connection
}

impl ManagerAuth {
    pub fn new() -> Result<Self, types::AnyError> {
        let conn = connect_to_database()?;
        Ok(Self { conn })
    }

    pub fn create(conn: Connection) {
        conn.execute(
            "
                CREATE TABLE IF NOT EXISTS managers (
                    token TEXT PRIMARY KEY,
                    state INTEGER
                )
            ",
            rusqlite::NO_PARAMS
        ).unwrap();
    }

    fn is_alphanumeric(&self, text: &str) -> bool {
        for ch in text.as_bytes() {
            if !(*ch as char).is_alphanumeric() {
                return false;
            }
        }

        true
    }

    fn into_state(&self, value: Value) -> Option<ManagerState> {
        match value {
            Value::Integer(i) => {
                match i {
                    0 => Some(ManagerState::Disabled),
                    1 => Some(ManagerState::Ok),
                    2 => Some(ManagerState::Admin),
                    _ => Some(ManagerState::Unknown)
                }
            },
            _ => None
        }
    }

    fn into_i16(&self, ms: ManagerState) -> u8 {
        match ms {
            ManagerState::Disabled => 0,
            ManagerState::Ok => 1,
            ManagerState::Admin => 2,
            ManagerState::Unknown => 3
        }
    }

    pub fn add_token(&self, token: &str, state: ManagerState) -> Result<(), types::AnyError> {
        // execute query
        let query = "INSERT INTO managers (token, state) VALUES (?1, ?2)";
        self.conn.execute(query, params![token, &self.into_i16(state)])?;
        Ok(())
    }

    pub fn update_state(&self, token: &str, state: ManagerState) -> Result<(), types::AnyError> {
        let query = "UPDATE managers SET state = ?1 WHERE token = ?2";
        self.conn.execute(query, params![self.into_i16(state), token])?;
        Ok(())
    }

    pub fn get_state(&self, token: &str)
        -> ManagerResult<ManagerState>
    {
        if token.len() != 32 || !self.is_alphanumeric(token) {
            return ManagerResult::Err("Invalid token")
        }

        // prepare query
        let query = "SELECT * FROM managers WHERE token = ?1";
        let mut stmt = self.conn.prepare(query).unwrap();

        // execute the query
        let mut results: Vec<Value> = Vec::new();
        let iter: _ = stmt.query_map(params![token], |row| row.get(1));

        // check errors
        if iter.is_err() {
            return ManagerResult::Err("Failed to query for token");
        }

        let iter = iter.unwrap();

        // put rows into vec
        for row in iter {
            if let Ok(row) = row {
                results.push(row);
            }
        }

        if results.len() != 0 {
            let res = results[0].clone();
            let state = self.into_state(res).unwrap();
            return ManagerResult::Ok(state);
        }

        ManagerResult::Err("Invalid token")
    }
}