// src/middleware/mod.rs
//
// WHY THIS EXISTS:
//   Middleware wraps every request/response — it's logic that executes before
//   and after your handlers without being in any specific handler.
//
//   Common use cases:
//     - Request tracing (add request ID, log duration)
//     - Authentication (verify JWT before allowing access)
//     - Rate limiting (reject requests that exceed N/second)
//     - CORS headers
//
//   V1 uses tower-http's TraceLayer for automatic request logging.
//   This is already wired into main.rs via the router.
//
// WHAT THE TRACE LAYER DOES:
//   For every request it logs:
//     - Method + path
//     - Status code
//     - Response time
//
//   Example log output:
//     INFO request{method=GET path=/products}: response status=200 latency=2.3ms
//
//   At scale, this is how you spot slow endpoints: tail logs, grep for high latency.
//
// FUTURE MIDDLEWARE (not yet implemented):
//   - request_id_middleware: assign UUID to each request for distributed tracing
//   - auth_middleware: validate JWT on protected routes
//   - rate_limit_middleware: token bucket per IP

// V1: Middleware is configured in main.rs via tower-http layers.
// This module is a placeholder for custom middleware functions we'll add in V2+.
