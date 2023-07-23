db:
    docker compose up -d

start: db
    cargo run

stop:
    docker compose stop

build:
    cargo build --release
