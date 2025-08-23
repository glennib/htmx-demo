set positional-arguments
set dotenv-load


CONTAINER_CMD := `(which docker > /dev/null 2>&1 && echo docker) || echo podman`

@_default:
  just --list

# Start the database container using Docker Compose
up:
  {{CONTAINER_CMD}} compose up --detach

# Stop and remove the database container using Docker Compose
down:
    {{CONTAINER_CMD}} compose down

# Connect to database with psql
psql *args:
   psql "$DATABASE_URL" "$@"


# Apply database migrations using sqlx
migrate:
    @# If the database is not running or the cloud instance is not forwarded, `sqlx migrate info` will fail
    sqlx migrate info
    @echo "Continue? (y/n)" && read -r response && if [ "$response" != "y" ]; then exit 1; fi
    sqlx migrate run

# Generate sea-orm entities
entities:
    rm -rf src/entity/entities
    sea generate entity \
        -o src/entity/entities \
        --with-prelude=all-allow-unused-imports \
        --with-serde=both \
        --with-copy-enums \
        --date-time-crate=chrono \
        --expanded-format
    cargo +nightly fmt
