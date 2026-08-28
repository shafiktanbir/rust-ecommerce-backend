#!/bin/bash
set -e

echo "Setting up PostgreSQL primary replication configuration..."

# Add replication host permission to pg_hba.conf
echo "host replication replicator 0.0.0.0/0 md5" >> "$PGDATA/pg_hba.conf"

# Create replication user
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    CREATE USER replicator WITH REPLICATION ENCRYPTED PASSWORD 'replicator_secret';
    SELECT pg_reload_conf();
EOSQL

echo "PostgreSQL primary replication initialized successfully."
