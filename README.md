# Kildin

A proxy rotator microservice written in Rust.

# Basic Usage

```
# Build the project
cargo build --release
cd target/release

# Create the sqlite db
sqlite3 database.db "CREATE TABLE IF NOT EXISTS managers (token TEXT PRIMARY KEY, state INTEGER)"

# Add yourself as the admin.
sqlite3 database.db "INSERT INTO managers (token, state) VALUES (\"YOUR UNIQUE TOKEN\", 2)"

# Create the config
touch config.toml
echo "[general]" >> config.toml
echo "database-path = \"test.db\"" >> config.toml
echo "[proxy-checker-settings]" >> config.toml
echo "pagination = 10" >> config.toml

# The request timeout for the proxy checker in seconds:
echo "timeout = 10" >> config.toml

# The interval (in seconds) that the rate limit checker waits.
echo "interval = 600" >> config.toml

# The maximum amount of times a proxy can fail until it's removed.
echo "max-fails = 20" >> config.toml

# The URL that the proxies use for testing:
echo "head-dest = \"https://duckduckgo.com/\"" >> config.toml

# Start the service.
./kildin config.toml &

# Add a proxy.
curl -XPOST -H "Content-type: application/json" -d '{"proxies": ["https://my-proxy-service.net:8000"]}' 'http://localhost:8000/proxies/add'

# Get a proxy with a minimum rating of 0.6, while telling the microservice the website you're using it for.
curl -XGET -H "Content-type: application/json" -d '{ "amount": 1, "website": "https://service.org/", "min_rating": 0.6 }' 'http://localhost:8000/proxies/get'
```
