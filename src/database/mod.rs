// tables:
// - proxies [index, schema, proxy address, rating]
// - managers (auth for managing the proxy) [token: text, state: num /0 = disabled, 1 = ok, 2 = admin/]

pub mod managers;
pub mod proxies;
pub mod ratelimited;