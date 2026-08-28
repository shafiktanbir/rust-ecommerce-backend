#!/bin/sh
set -e

echo "Waiting for primary postgres server to become ready..."
until pg_isready -h postgres -p 5432 -U "${POSTGRES_USER:-ecommerce}"; do
  echo "Primary postgres not ready yet. Retrying in 1 second..."
  sleep 1
done

if [ ! -s "$PGDATA/PG_VERSION" ]; then
  echo "Initializing streaming replication basebackup from primary..."
  rm -rf "${PGDATA:?}"/*
  PGPASSWORD=replicator_secret pg_basebackup -h postgres -p 5432 -U replicator -D "$PGDATA" -Fp -Xs -R
  chmod 700 "$PGDATA"
  echo "Streaming replica basebackup completed."
fi

echo "Starting PostgreSQL read replica..."
exec docker-entrypoint.sh postgres
