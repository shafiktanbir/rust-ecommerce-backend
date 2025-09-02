# Post 030: Building a Production-Ready Axum Health Check Endpoint (`/health` & `/health/db`)
* **Target Audience**: DevOps Engineers, SRE Leads, Kubernetes Operators.
* **Viral Hook**: "Why your `/health` endpoint returning `200 OK` when the database is dead will trick Kubernetes into sending traffic to broken pods."
* **Technical Code**:
  ```rust
  pub async fn health_db(State(state): State<Arc<AppState>>) -> Result<impl IntoResponse> {
      sqlx::query("SELECT 1").execute(&state.db.writer).await?;
      Ok((StatusCode::OK, Json(json!({"status": "healthy", "database": "connected"}))))
  }
  ```
* **Metrics Impact**: 100% accurate pod readiness routing; zero traffic misrouted to pods experiencing database connectivity failures.
* **Cold Outreach DM**: "Hey [Name], saw [Company] is refining Kubernetes health probes. Separating application liveness (`/health`) from DB readiness (`/health/db`) prevents K8s Ingress from sending traffic to pods with lost database pools. Shared our health check implementation here!"

---
