# Post 076: Implementing Idempotent Transactions with Unique Constraint Violations (`23505`)
* **Target Audience**: Backend Developers, API Engineers.
* **Viral Hook**: "How to handle duplicate order submission retries without separate `SELECT` queries by catching PostgreSQL error code `23505`."
* **Technical Code**:
  ```rust
  match sqlx::query!("INSERT INTO orders (id, user_id) VALUES ($1, $2)", order_id, user_id)
      .execute(&pool).await {
          Ok(_) => Ok(StatusCode::CREATED),
          Err(sqlx::Error::Database(db_err)) if db_err.code().as_deref() == Some("23505") => {
              // Unique constraint violation (Duplicate submission ignored safely!)
              Ok(StatusCode::OK)
          }
          Err(err) => Err(err.into()),
      }
  ```
* **Metrics Impact**: Ignored 100% of duplicate API retries; database roundtrips reduced from 2 to **1** per request.
* **Cold Outreach DM**: "Hey [Name], saw your post on API idempotency. Catching PostgreSQL unique constraint violation error `23505` directly inside insert queries handles duplicate retries atomically in 1 SQL statement. Shared our error handling snippet here!"

---
