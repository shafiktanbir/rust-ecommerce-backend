-- migrations/002_v5_indexes.sql
--
-- Milestone V5 Composite Index Optimizations
--

-- Composite index on products (name, price) for catalog filtering & sorting
CREATE INDEX IF NOT EXISTS idx_products_name_price ON products(name, price);

-- Composite index on orders (user_id, status, created_at DESC) for user dashboard history lookups
CREATE INDEX IF NOT EXISTS idx_orders_user_status ON orders(user_id, status, created_at DESC);
