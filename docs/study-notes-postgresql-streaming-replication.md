# Study Notes: PostgreSQL Physical Streaming Replication & Read-Write Splitting (CQRS)

> **Workspace**: `rust ecommerse-loop`
> **Topic**: PostgreSQL Streaming Replication, Basebackup, WAL Receiver/Sender, CQRS Dual Pools, Replication Lag Circuit Breaker, and Cloud Deployment Differences.

---

## 1. Fundamentals: Why Replication & CQRS?

Under high application traffic (e.g. 10,000 req/sec), single database instances bottleneck on CPU, disk I/O, and lock contention between heavy write transactions (`INSERT`/`UPDATE`) and high-volume read queries (`SELECT`).

### Read-Write Splitting (CQRS Architecture)
* **Primary (Writer) Database**: Handles **100% of Writes** (`INSERT`, `UPDATE`, `DELETE`, transactions).
* **Replica (Reader) Database**: A read-only clone handling **100% of Reads** (`GET`).

```
                    ┌───────────────────────────┐
                    │      Rust Backend App     │
                    └─────────────┬─────────────┘
                                  │
            ┌─────────────────────┴─────────────────────┐
            │ Writes (POST/PUT/DELETE)                  │ Reads (GET)
            ▼                                           ▼
┌───────────────────────┐   WAL Stream TCP    ┌───────────────────────┐
│ Primary Database (DB) │────────────────────►│  Replica Database     │
│ (Port 5432 - Writer)  │  (Real-time Copy)   │ (Port 5435 - Reader)  │
└───────────────────────┘                     └───────────────────────┘
```

---

## 2. Under the Hood: Write-Ahead Logging (WAL) & Streaming

PostgreSQL replication works via **Write-Ahead Logging (WAL)**:
1. Every write transaction is appended to a binary journal (WAL) on the Primary disk before altering table data.
2. The Primary database runs a `walsender` process that streams raw WAL bytes over a persistent TCP socket.
3. The Replica database runs a `walreceiver` process that catches incoming WAL bytes and replays them into its local tables in real time.

---

## 3. Primary Replication Initializer Script Breakdown

File: [`scripts/init-primary-replication.sh`](file:///home/shafikul/Documents/coding/research-playground-loop/rust%20ecommerse-loop/scripts/init-primary-replication.sh)
*Mounted to `/docker-entrypoint-initdb.d/` on Primary container.*

```bash
#!/bin/bash
set -e

# 1. Add firewall rule to pg_hba.conf allowing streaming replication from any host
echo "host replication replicator 0.0.0.0/0 md5" >> "$PGDATA/pg_hba.conf"

# 2. Create dedicated replication user with REPLICATION privilege & reload config
psql -v ON_ERROR_STOP=1 --username "$POSTGRES_USER" --dbname "$POSTGRES_DB" <<-EOSQL
    CREATE USER replicator WITH REPLICATION ENCRYPTED PASSWORD 'replicator_secret';
    SELECT pg_reload_conf();
EOSQL
```

### Granular Breakdown:
* `set -e`: Halts script execution immediately if any command exits with an error code.
* `pg_hba.conf`: Host-Based Authentication file (PostgreSQL's firewall).
  * `host`: TCP/IP network connection.
  * `replication`: Special mode allowing streaming WAL logs (not standard SQL).
  * `replicator`: User allowed to connect.
  * `0.0.0.0/0`: IP range (allows all containers on Docker bridge).
  * `md5`: Authentication method.
* `-v ON_ERROR_STOP=1`: Forces `psql` to abort if any SQL query fails.
* `<<-EOSQL ... EOSQL`: Bash Heredoc passing multi-line SQL into `psql`. The `-` strips leading tab indents.
* `WITH REPLICATION`: Gives the user authorization to connect to PostgreSQL's binary replication socket.
* `SELECT pg_reload_conf();`: Tells PostgreSQL engine to re-parse `pg_hba.conf` in memory live without restarting.

---

## 4. Replica Startup Script Breakdown

File: [`scripts/start-replica.sh`](file:///home/shafikul/Documents/coding/research-playground-loop/rust%20ecommerse-loop/scripts/start-replica.sh)
*Entrypoint script for `postgres_replica` container.*

```bash
#!/bin/sh
set -e

# 1. Wait for Primary DB to be accepting TCP connections
until pg_isready -h postgres -p 5432 -U "${POSTGRES_USER:-ecommerce}"; do
  sleep 1
done

# 2. First-time initialization check
if [ ! -s "$PGDATA/PG_VERSION" ]; then
  rm -rf "${PGDATA:?}"/*
  PGPASSWORD=replicator_secret pg_basebackup -h postgres -p 5432 -U replicator -D "$PGDATA" -Fp -Xs -R
  chmod 700 "$PGDATA"
fi

# 3. Start PostgreSQL server process as PID 1
exec docker-entrypoint.sh postgres
```

### Granular Breakdown:
* `pg_isready -h postgres -p 5432`: Probes Primary host over TCP until it is ready.
* `[ ! -s "$PGDATA/PG_VERSION" ]`: Checks if data directory is empty (first-time boot).
* `${PGDATA:?}`: Safety check! Aborts if `$PGDATA` is empty/null to prevent running `rm -rf /*`.
* `pg_basebackup`:
  * `-h postgres -p 5432 -U replicator`: Connects to Primary as user `replicator`.
  * `-D "$PGDATA"`: Target directory for cloned data.
  * `-Fp`: Plain format (writes directories directly).
  * `-Xs`: Streams WAL logs concurrently during backup download.
  * **`-R` (Crucial Flag)**: Automatically creates `standby.signal` and writes `primary_conninfo` into `postgresql.auto.conf`, instructing PostgreSQL to start as a Read-Only Replica!
* `chmod 700 "$PGDATA"`: Restricts directory permissions (PostgreSQL requirement).
* `exec docker-entrypoint.sh postgres`: Replaces script shell with `postgres` process (PID 1) so Docker shutdown signals (`SIGTERM`) are handled gracefully.

---

## 5. How PostgreSQL Runs the 24/7 Streaming Loop Automatically

Once PostgreSQL boots with `standby.signal` present:
1. **`walreceiver` Process**: Launched on Replica. Opens a persistent TCP socket to Primary using `primary_conninfo`.
2. **`walsender` Process**: Launched on Primary to serve the Replica.
3. **Continuous Loop**: Whenever a write hits Primary, `walsender` pushes WAL binary chunks to `walreceiver`, which writes them to disk and hands them to the `startup/redo` process to replay changes into tables. If network drops, `walreceiver` continuously retries connecting.

---

## 6. Rust Application Implementation (Dual Pools & Circuit Breaker)

### Dual Connection Pools (`src/db/mod.rs`)
```rust
pub struct DbPools {
    pub writer: PgPool, // Points to Primary (Port 5432)
    pub reader: PgPool, // Points to Replica (Port 5435)
}
```

### Replication Lag Circuit Breaker (`src/main.rs`)
To prevent users from seeing stale data when replication falls behind:
* Tokio spawns a 1-second background interval loop querying `pg_stat_replication`.
* **Trip Condition (> 500ms lag)**: Toggles `is_replica_lagging` atomic flag to `true` $\rightarrow$ Axum routes 100% of read traffic back to Primary.
* **Recovery Condition (< 100ms lag)**: Clears atomic flag to `false` $\rightarrow$ Resumes Replica reads.

---

## 7. Cloud Production Deployment Differences (Local Docker vs Cloud)

| Setting | Local Docker | Cloud Production (AWS / Hetzner / GCP) |
| :--- | :--- | :--- |
| **Target Host (`-h`)** | `postgres` (Docker DNS) | **Private IP** (`10.0.1.50`) or Internal DNS (`primary.db.internal`) |
| **Network Security** | Docker internal bridge | **VPC Private Subnet** + Security Groups (Port 5432 open ONLY to Replica IP) |
| **Transport Security** | Unencrypted TCP | **TLS/SSL Encrypted (`hostssl` / `sslmode=require`)** |
| **Authentication** | Hardcoded secret string | Loaded from Secrets Manager / Vault (`$REPLICATOR_PASSWORD`) |
| **Auth Hash** | `md5` | `scram-sha-256` |

### Cloud VPC Security Rules
1. **Never assign Public IPs** to Primary or Replica database instances.
2. **Restrict Firewall Ingress**: Open port 5432 on Primary ONLY to the specific Private IP of the Replica container/VM (`10.0.2.99/32`).
3. **Enforce SSL/TLS**: Use `hostssl` in `pg_hba.conf` and `sslmode=require` in connection strings to encrypt WAL streams across cloud network switches.

---

## 8. Read-Your-Own-Writes (Read-After-Write) Sticky Session Pattern

### The Problem
If a user creates an order (`POST /orders`) or updates their profile and immediately gets redirected to `GET /orders`, asynchronous replication lag (even 10ms) could cause the Replica DB to return an empty array or 404.

### The Solution: Redis-Backed Sticky Sessions
1. **On Write Mutation**: We write a short-lived key in Redis: `sticky_primary:{user_id}` with a **5-second TTL**.
2. **On Read Query**: `AppState::get_reader_pool_for_user(Some(user_id))` checks Redis:
   - **If `sticky_primary:{user_id}` exists**: Route read directly to **Primary DB (Writer Pool)**.
   - **If key does not exist**: Route read to **Read Replica DB (Reader Pool)**.

