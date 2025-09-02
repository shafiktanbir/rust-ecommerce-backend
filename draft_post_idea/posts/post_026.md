# Post 026: Preventing Memory Leaks in Long-Running Tokio Background Tasks
* **Target Audience**: Lead SREs, Systems Developers, Backend Architects.
* **Viral Hook**: "Why our Tokio background worker RAM grew from 20MB to 1.8GB over 48 hours — and how an un-bounded `mpsc::channel` was the culprit."
* **Core Problem**: Using `mpsc::unbounded_channel()` allows message buffers to grow infinitely if producers push messages faster than the consumer worker can process them.
* **Technical Code**:
  ```rust
  // ❌ BAD: Memory leak risk under consumer lag
  let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

  // ✅ GOOD: Bounded backpressure channel (caps buffer at 1,000 items)
  let (tx, rx) = tokio::sync::mpsc::channel(1000);
  ```
* **Metrics Impact**: RAM usage stabilized at a flat **22MB** over 7-day continuous soak testing; backpressure properly throttles fast producers.
* **Cold Outreach DM**: "Hey [Name], saw your post on Tokio background worker stability. Switching from unbounded channels to bounded `mpsc::channel(1000)` applies memory backpressure and prevents slow consumer lag from causing OOMKills in Kubernetes. Shared our channel tuning guide here!"

---
