// crate
use crate::database::proxies::{Proxies, Proxy};
use crate::helpers::config::ProxyCheckerSettings;
use crate::helpers::{logger::{Level, Logger}, types};

// reqwest
use reqwest::blocking::Client;
use reqwest::Proxy as ReqProxy;

// std
use std::thread;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

macro_rules! proxy_log {
    ($logger:expr, $success:expr, $verb:expr, $amount:expr) => {
        {
            let success = if $success { "Successfully" } else { "Couldn't" };
            let verb = if $success { format!("{}d", $verb) } else { $verb.into() };
            let level = if $success { Level::Info } else { Level::Warn };
            let msg = format!("ProxyManager: {} {} {} proxies.", success, verb, $amount);
            $logger.log(level, &msg);
        }
    };
}

pub struct ProxyChecker {
    proxies: Proxies,
    pcs: ProxyCheckerSettings,
    logger: Logger,
}

impl ProxyChecker {
    pub fn new(pcs: ProxyCheckerSettings, logger: Logger) -> Self {
        let proxies = Proxies::new().expect("Couldn't connect to database");
        Self { proxies, pcs, logger }
    }

    fn now() -> u64 {
        let start = SystemTime::now();
        let dur = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        dur.as_secs()
    }

    fn check_proxy(pcs: ProxyCheckerSettings, proxy: &(u32, Proxy))
        -> Result<Proxy, reqwest::Error>
    {
        let dur = Duration::from_secs(pcs.timeout);
        let before = Self::now();

        // construct client
        let proxy_addr = format!("{}://{}:{}", proxy.1.schema, proxy.1.address, proxy.1.port);
        let client = Client::builder()
            .proxy(ReqProxy::https(&proxy_addr)?)
            .timeout(dur).build()?;

        // send req and calculate score
        client.head(&pcs.dest).send()?;

        // score
        let new_score = (dur.as_secs() - (Self::now() - before)) as f64;
        let score = (new_score + proxy.1.rating) / 2.0;

        Ok(Proxy {
            schema: proxy.1.schema.clone(),
            address: proxy.1.address.clone(),
            port: proxy.1.port.clone(),
            rating: if score > 10.0 { 10.0 } else { score },
            fails: proxy.1.fails,
            blacklisted: false
        })
    }

    pub fn update(&mut self) -> Result<bool, types::AnyError> {
        let mut idx = 0;

        loop {
            let mut proxies = Vec::new();
            let mut handles = Vec::new();
            let entries: _ = self.proxies.after(idx, self.pcs.pagination)?;

            if entries.len() == 0 {
                return Ok(false);
            }

            // set idx
            idx = entries[entries.len() - 1].0;

            // launch threads
            for proxy in entries.clone() {
                if proxy.1.blacklisted {
                    continue;
                }

                let pcs = self.pcs.clone();
                let handle: _ = thread::spawn(move || Self::check_proxy(pcs, &proxy));
                handles.push(handle);
            }

            // wait for threads to finish
            for handle in handles {
                let res = handle.join();

                if res.is_err() {
                    continue;
                }

                if let Ok(proxy) = res.unwrap() {
                    let mut proxy = proxy.clone();

                    if proxy.rating != 0.0 {
                        proxy.fails -= 1;
                    }

                    proxies.push(proxy);
                }
            }

            for (_, entry) in entries {
                let e = entry.clone();
                let inner_proxies = proxies.iter()
                    .map(|p| (p.address.clone(), p.port))
                    .collect::<Vec<(String, u16)>>();

                if !inner_proxies.contains(&(e.address, e.port)) {
                    let mut proxy = entry.clone();
                    proxy.fails += 1;
                    proxies.push(proxy);
                }
            }

            for (idx, proxy) in proxies.clone().iter().enumerate() {
                proxies[idx].blacklisted = proxy.fails >= self.pcs.max_fails;
            }

            // try to update proxies
            let amount = proxies.len();
            let success = self.proxies.update_proxies(proxies).is_ok();
            proxy_log!(self.logger, success, "update", amount);
            
            return Ok(true);
        }
    }
}
