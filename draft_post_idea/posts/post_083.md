# Post 083: Benchmarking Lock Retention Times Across Different Network Latency Links
* **Target Audience**: Lead SREs, Systems Architects.
* **Viral Hook**: "How 20ms network latency between your API worker and PostgreSQL database increases row lock retention times by 400%."
* **Core Problem**: Lock retention time is directly proportional to network roundtrip latency ($W = \text{Network RTT} + \text{Query Time}$). Moving API workers away from DB nodes explodes lock queues!
* **Metrics Impact**: Co-located API workers and DB nodes in the same private VPC subnet (`< 0.5ms` RTT), reducing lock hold times by **85%**.
* **Cold Outreach DM**: "Hey [Name], saw your post on multi-region cloud latency. Co-locating API workers and PostgreSQL primary nodes in the same private VPC subnet minimizes network RTT and keeps row lock hold times under 5ms. Shared our network benchmark data here!"

---
