# Post #1: The 504 Gateway Timeout Hidden Inside Linux Networking: How a Sysctl Flag Nearly Killed Our 3,000 VU Benchmark

## 📌 Executive Meta-Summary
* **Target Audience**: Founders, CTOs, Head of Infrastructure, Lead SREs.
* **Core Problem**: Mysterious 504 Gateway Timeouts & 502 Bad Gateways on dual-homed cloud instances (public IP + private VPC) despite low CPU/RAM usage.
* **Technical Root Cause**: Linux Strict Reverse Path Filtering (`rp_filter = 1`) dropping asymmetric return packets on secondary Netplan interfaces (`enp7s0`).
* **Hardware & Cost**: 4x Hetzner `cx23` (2 vCPU, 4GB RAM) @ €20.48/month (~$22.50/mo).
* **Metrics Impact**: Jumped from 0 RPS (crashing connections) to **4,931.11 RPS (324,951 requests, 100% success rate)** under 3,000 concurrent Virtual Users (VUs).
* **Outreach Hook**: "Are your AWS/Hetzner instances dropping 504s under load despite low CPU? Check your kernel sysctl routing table before paying for bigger servers."

---

## 📱 Social Post Draft (LinkedIn / X / Engineering Blog)

### 🚨 Hook
We spun up a 4-node Rust microservice cluster on Hetzner Cloud for €20/month. 
We fired 3,000 concurrent users (k6) at it... and instantly got slammed with **HTTP 504 Gateway Timeouts**.

CPU usage on our Axum workers was under 15%. 
RAM was barely touching 120MB. 
Yet Nginx was sitting there timing out after 60 seconds.

Here is how a single Linux kernel setting (`rp_filter`) nearly killed our benchmark — and how we fixed it to hit **4,931 RPS with 0.00% errors**. 👇

---

### 🔍 The Incident & Diagnostic Trace

When running multi-node cloud clusters with Nginx load balancing:
- Public IP attached to `eth0` (External traffic).
- Private VPC (`10.0.1.0/24`) attached to secondary interface `enp7s0` (Internal DB/API traffic).

Here is what was happening under the hood:

```
❌ ASYMMETRIC ROUTING PACKET DROP:
Nginx LB (10.0.1.10) ──── (enp7s0 IN) ───► Worker Node (10.0.1.11)
Nginx LB (10.0.1.10) ◄── (eth0 OUT - DROPPED BY rp_filter=1) ──┘
```

1. Nginx sends TCP SYN over the private VPC interface (`enp7s0`).
2. The Axum API worker receives the packet, processes the request in < 2ms, and formats the response.
3. The Linux Kernel checks its routing table. Because the default route (`0.0.0.0/0`) points out `eth0`, Linux attempts to return the reply out `eth0`.
4. **Strict Reverse Path Filtering (`rp_filter = 1`)** detects that the incoming packet arrived on `enp7s0` but the return path points to `eth0`.
5. Linux silently drops the packet as suspicious! Nginx waits, times out, and throws HTTP 504.

---

### 🛠️ The SRE Fix

We injected two critical network configurations into our Terraform `user_data` boot script:

1. **Netplan VPC Auto-Configuration (`/etc/netplan/60-vpc.yaml`)**:
   Prevent secondary interface `enp7s0` from staying in `state DOWN`.
   ```yaml
   network:
     version: 2
     ethernets:
       enp7s0:
         dhcp4: true
   ```

2. **Switch to Loose Reverse Path Filtering (`rp_filter = 2`)**:
   ```bash
   # Enable loose RP filtering across all interfaces
   sysctl -w net.ipv4.conf.all.rp_filter=2
   sysctl -w net.ipv4.conf.default.rp_filter=2
   sysctl -w net.ipv4.conf.enp7s0.rp_filter=2
   ```

---

### 📊 Empirical Results After Fix

* **Peak Throughput**: **4,931.11 Requests/Sec (RPS)**
* **Total Requests**: **324,951 requests in 65 seconds**
* **Error Rate**: **0.00% (0 failed requests out of 324,951)**
* **Concurrency**: **3,000 Virtual Users (VUs)**
* **Hardware Cost**: **€20.48 / month ($22.50/mo)**

---

### 💼 Business Takeaway for Founders & CTOs
Don't scale your AWS/Hetzner bill when latency spikes under heavy load. Most 504 errors in multi-node private subnets aren't application performance bugs — they are kernel-level asymmetric routing packet drops.

Fixing sysctl configs costs $0 and scales your existing infra by 10x.

---

## 🎯 Cold Outreach Conversion Script

**Target Persona**: CTOs / Head of Infra at mid-sized SaaS or E-commerce platforms using AWS VPC or Hetzner.

**Message**:
> "Hey [Name], noticed [Company] is scaling up your backend infrastructure recently. 
> 
> We recently debugged a edge-case issue where multi-homed cloud nodes were throwing silent 504 timeouts under 3,000 VU load despite 15% CPU utilization due to Linux kernel `rp_filter` asymmetric packet drops. We were able to unlock 4,900+ RPS on €20/mo hardware with zero hardware upgrades.
> 
> Wrote a breakdown of the diagnostic trace here: [Link to post]. If you're ever auditing VPC routing or scaling backend throughput, happy to share our Terraform & sysctl configs!"
