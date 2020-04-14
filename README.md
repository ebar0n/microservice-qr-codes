# microservice-qr-codes

# Installation

```bash
# Build
docker-compose build

# Start
docker-compose run --rm --service-ports api bash

# Run server 8000
cargo watch -x run
```

## Start server

```bash
docker-compose up -d api
```

# Test

## cargo fmt
```bash
# Execute code style
cargo +nightly fmt

# Execute check code style
cargo +nightly fmt --all -- --check
```

## cargo test
```bash
# Execute test
cargo test
```


# Documentación

## Health check
```bash
curl -i -X GET "http://127.0.0.1:8000/health/"

{"health":"Ok","agent":"[agent]","created_at":"[time]","version":"0.0.1"}
```

## Generate QR

```bash
curl -i -X GET "http://127.0.0.1:8000/api/v1/generate/?message=hello+wold"

{"url":"/static/[token].png","message":""}
```