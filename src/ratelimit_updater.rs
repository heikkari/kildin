// crate
use crate::database::ratelimited::RateLimited;
use crate::database::ratelimited::RateLimitEntry;
use crate::helpers::{logger::{Level, Logger}, types};

// rayon
use rayon::prelude::*;

// std
use std::time::{SystemTime, UNIX_EPOCH};

pub struct RatelimitUpdater {
    ratelimited: RateLimited, // ratelimited proxies
    logger: Logger
}

impl RatelimitUpdater {
    pub fn new(logger: Logger) -> Self {
        let ratelimited = RateLimited::new()
            .expect("Couldn't connect to database");
        Self { ratelimited, logger }
    }

    fn now() -> u64 {
        let start = SystemTime::now();
        let dur = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        dur.as_secs()
    }


    pub fn update(&mut self) -> Result<(), types::AnyError> {
        let proxies: _ = self.ratelimited.read_ratelimited(100,
            |entries| {
                // get results from proxy testing
                let rle: _ = entries.par_iter()
                    .filter(|e: _| (Self::now() as i64 - e.1.until as i64) >= 0)
                    .map(|e| e.1.clone())
                    .collect::<Vec<RateLimitEntry>>();

                let msg = format!("RateLimitManager: Found {} proxies!", rle.len());
                self.logger.log(Level::Info, &msg);
                rle
            }
        )?;

        let len = proxies.len();

        match self.ratelimited.remove(proxies) {
            Ok(_) => {
                let msg = format!("RateLimitManager: Successfully unratelimited {} proxies!", len);
                self.logger.log(Level::Info, &msg);
            },
            Err(why) => {
                let msg = format!("RateLimitManager: Error {:?}", why);
                self.logger.log(Level::Info, &msg);
            }
        };

        Ok(())
    }
}