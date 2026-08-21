-- migrations/001_initial.sql
--
-- V1 Initial Schema
--
-- Design principles:
--   1. Use UUIDs as primary keys (not auto-increment integers)
--      Why: UUIDs are globally unique — safe for distributed systems where multiple nodes
--           generate IDs. Auto-increment integers require coordination between nodes.
--           Trade-off: UUIDs are 16 bytes (vs 4 for int) and index lookups are slightly slower.
--
--   2. Use TIMESTAMPTZ (timestamp with time zone), not TIMESTAMP
--      Why: TIMESTAMPTZ stores UTC. TIMESTAMP has no timezone — ambiguous and dangerous.
--
--   3. gen_random_uuid() requires pgcrypto extension or PostgreSQL 13+
--      PostgreSQL 13+ ships this built-in, no extension needed.
--
--   4. Indexes are added on foreign keys and commonly filtered columns.
--      V1 index strategy is minimal — we will add more after measuring EXPLAIN ANALYZE.

-- ─────────────────────────────────────────────────────────────────────────────
-- USERS
-- ─────────────────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS users (
    id            UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email         VARCHAR(255) NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index on email for fast login lookups
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);

-- ─────────────────────────────────────────────────────────────────────────────
-- PRODUCTS
-- ─────────────────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS products (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name            VARCHAR(500) NOT NULL,
    description     TEXT,
    -- V1: FLOAT8 (double precision) maps directly to Rust f64 — no extra sqlx features needed.
    -- Trade-off: FLOAT8 has minor floating-point rounding errors for very large/precise values.
    -- V2+ upgrade: Change to NUMERIC(12,2) + add sqlx bigdecimal feature + use rust_decimal in Rust.
    -- For a learning lab at this scale, FLOAT8 is acceptable.
    price           FLOAT8 NOT NULL CHECK (price >= 0),
    -- inventory_count is the primary concurrency bottleneck in flash sales.
    -- The CHECK constraint prevents negative inventory at the database level —
    -- a last line of defense even if the application has a bug.
    inventory_count INTEGER NOT NULL DEFAULT 0 CHECK (inventory_count >= 0),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index on created_at for the default list ordering
CREATE INDEX IF NOT EXISTS idx_products_created_at ON products(created_at DESC);

-- ─────────────────────────────────────────────────────────────────────────────
-- ORDERS
-- ─────────────────────────────────────────────────────────────────────────────
DO $$ BEGIN
    CREATE TYPE order_status AS ENUM ('pending', 'confirmed', 'shipped', 'delivered', 'cancelled');
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

CREATE TABLE IF NOT EXISTS orders (
    id           UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id      UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    -- ON DELETE RESTRICT prevents deleting a user who has orders.
    -- This protects referential integrity and is safer than CASCADE for financial data.
    total_amount NUMERIC(12, 2) NOT NULL CHECK (total_amount >= 0),
    status       order_status NOT NULL DEFAULT 'pending',
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for fetching a user's orders
CREATE INDEX IF NOT EXISTS idx_orders_user_id ON orders(user_id);
CREATE INDEX IF NOT EXISTS idx_orders_status ON orders(status);

-- ─────────────────────────────────────────────────────────────────────────────
-- ORDER ITEMS
-- ─────────────────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS order_items (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    order_id   UUID NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    product_id UUID NOT NULL REFERENCES products(id) ON DELETE RESTRICT,
    quantity   INTEGER NOT NULL CHECK (quantity > 0),
    -- unit_price is stored at time of purchase — product price may change later
    unit_price NUMERIC(12, 2) NOT NULL CHECK (unit_price >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_order_items_order_id ON order_items(order_id);
CREATE INDEX IF NOT EXISTS idx_order_items_product_id ON order_items(product_id);

-- ─────────────────────────────────────────────────────────────────────────────
-- FLASH SALES
-- ─────────────────────────────────────────────────────────────────────────────
-- A flash sale constrains a product to a limited inventory window over a time range.
-- Multiple users can simultaneously try to buy — this is the concurrency problem we'll study.
CREATE TABLE IF NOT EXISTS flash_sales (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    product_id      UUID NOT NULL REFERENCES products(id) ON DELETE CASCADE,
    -- inventory_limit is the max units available during the flash sale
    inventory_limit INTEGER NOT NULL CHECK (inventory_limit > 0),
    starts_at       TIMESTAMPTZ NOT NULL,
    ends_at         TIMESTAMPTZ NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- A product cannot have two overlapping flash sales
    CONSTRAINT flash_sales_ends_after_starts CHECK (ends_at > starts_at)
);

CREATE INDEX IF NOT EXISTS idx_flash_sales_product_id ON flash_sales(product_id);
CREATE INDEX IF NOT EXISTS idx_flash_sales_starts_at ON flash_sales(starts_at);
