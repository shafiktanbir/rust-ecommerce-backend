# 📘 Systems Engineering & SRE Study Guide: Converting RPS to Real-World User Scale

This study guide explains how to convert backend system throughput (**RPS - Requests Per Second**) into real-world business metrics (**Peak Concurrent Users**, **DAU - Daily Active Users**, and **MAU - Monthly Active Users**). 

Use this handbook for **System Design Interviews**, **SRE capacity planning**, and **Investor/CTO pitch presentations**.

---

## 🧭 Table of Contents
1. [Where Does 86,400 Come From?](#1-where-does-86400-come-from)
2. [The Plain-Text Mathematical Formulas](#2-the-plain-text-mathematical-formulas)
3. [Step-by-Step Worked Example (4,000 RPS → 6.9M DAU)](#3-step-by-step-worked-example-4000-rps--69m-dau)
4. [Capacity Planning for 1 Million Users](#4-capacity-planning-for-1-million-users)
5. [The CTO & Founder Investor Pitch Guide](#5-the-cto--founder-investor-pitch-guide)
6. [Summary Reference Matrix](#6-summary-reference-matrix)
7. [Self-Test Practice Problems & Interview Flashcards](#7-self-test-practice-problems--interview-flashcards)

---

## 1. Where Does 86,400 Come From?

`86,400` is the total number of seconds in one full 24-hour day:

* **1 minute** = 60 seconds
* **1 hour** = 60 minutes × 60 seconds = **3,600 seconds**
* **1 day** = 24 hours × 3,600 seconds = **86,400 seconds**

If your backend server processes **4,000 requests every single second**, in 24 hours (1 full day) it processes:  
`4,000 requests/sec × 86,400 seconds/day = 345,600,000 requests per day (345.6 Million requests)`.

---

## 2. The Plain-Text Mathematical Formulas

### Formula 1: Daily Request Capacity
> **Total Daily Requests = RPS × 86,400 seconds/day**

### Formula 2: Daily Active Users (DAU) - Flat Traffic
> **DAU = Total Daily Requests ÷ Average Requests per User per Day**

> 💡 *Note*: In standard e-commerce and consumer web apps, an average active user makes **15 to 25 API requests** per daily session (login, search, view 10 products, view cart, checkout).

### Formula 3: Daily Active Users (DAU) - Peak Traffic Adjusted
> **Peak-Adjusted DAU = DAU ÷ Peak Factor (typically 2.0 to 3.0)**

> 💡 *Note*: Traffic is not flat over 24 hours. Daytime peak hours usually process **2.5×** the average hourly rate.

### Formula 4: Peak Concurrent Users (Simultaneous Live Users)
> **Concurrent Users = RPS × Think Time (seconds between user actions)**

> 💡 *Note*: Users do not click every millisecond. Average **Think Time** (reading, looking at photos) is **5 to 10 seconds**.

### Formula 5: Required RPS for Target DAU
> **Average RPS = (Target DAU × Requests per User) ÷ 86,400**  
> **Peak RPS Required = Average RPS × Peak Multiplier (2.5×)**

### Formula 6: Required RPS for Target Concurrent Users
> **Required RPS = Target Concurrent Users ÷ Think Time (seconds)**

---

## 3. Step-by-Step Worked Example (4,000 RPS → 6.9M DAU)

Using a baseline benchmark result of **4,000 RPS**:

### Step 1: Total Daily Capacity
`4,000 RPS × 86,400 seconds/day = 345,600,000 requests per day (345.6M)`

### Step 2: Convert to Daily Active Users (DAU)
Assuming **20 requests per user per day**:
* **Flat DAU Capacity**: `345,600,000 ÷ 20 = 17,280,000 Daily Active Users (17.28M DAU)`

### Step 3: Account for Daytime Peak Spikes (2.5× Peak Multiplier)
* **Sustained Peak-Safe DAU Capacity**: `17,280,000 ÷ 2.5 = 6,912,000 Daily Active Users (~6.91M DAU)`

### Step 4: Peak Concurrent Users (Simultaneous Live Users on Site)
* **At 5-second Think Time**: `4,000 × 5 = 20,000 active users simultaneously`
* **At 10-second Think Time**: `4,000 × 10 = 40,000 active users simultaneously`

---

## 4. Capacity Planning for 1 Million Users

### Scenario A: Target is 1 Million Daily Active Users (1M DAU)
Assuming 20 requests/user/day and a 2.5× peak daytime factor:

1. **Total Daily Volume**: `1,000,000 users × 20 reqs/user = 20,000,000 requests/day`
2. **Average RPS**: `20,000,000 ÷ 86,400 = 231.48 RPS`
3. **Peak RPS Required**: `231.48 × 2.5 = ~578.7 Peak RPS`

> 💡 *Takeaway*: To serve **1 Million Daily Active Users**, your backend system only needs to sustain **~580 Peak RPS**.

---

### Scenario B: Target is 1 Million Concurrent Users (1M Simultaneous Online Users)
Assuming 1,000,000 users active on the site at the exact same moment with a 5-second think time between clicks:

* **Required RPS** = `1,000,000 users ÷ 5 seconds = 200,000 RPS`

> 💡 *Takeaway*: To support **1 Million simultaneous live users** clicking every 5 seconds, your system must handle **200,000 RPS**.

---

## 5. The CTO & Founder Investor Pitch Guide

When an investor, client, or senior tech interviewer asks: **"How many users can your system handle?"** or **"Do you have scaling experience?"**, use this structured response:

### 🗣️ The Investor Pitch Script:

> *"Our backend infrastructure has been benchmarked to sustain **4,000 requests per second (RPS)** with **0% error rate** and sub-200ms latency on baseline cloud instances.
>
> In business metrics, this capacity supports:
> 1. **Peak Flash Sale Traffic**: **20,000 to 40,000 users browsing simultaneously** during peak traffic.
> 2. **Daily Active Scale**: Over **345 Million requests per day**, supporting **6.9 Million Daily Active Users (DAU)** even during peak daytime traffic spikes.
> 3. **Total Platform Scale**: Over **30 Million to 50+ Million Monthly Active Users (MAU)**.
>
> Because our architecture is horizontally scalable (stateless API services + connection pooling), scaling to 10,000+ RPS simply requires deploying additional worker nodes with zero code changes."*

---

## 6. Summary Reference Matrix

| Metric | Empirical Result | Business User Equivalent | Formula Used |
| :--- | :--- | :--- | :--- |
| **Raw Throughput** | `4,000 RPS` | **345,600,000 Requests / Day** | `RPS × 86,400s` |
| **Flash Sale Peak** | 4,000 RPS (0% error) | **20,000 - 40,000 Concurrent Users** | `RPS × Think Time (5-10s)` |
| **Daily Active Scale** | 20 req/user/day | **6.91 Million Peak DAU** | `(Daily Requests ÷ 20) ÷ 2.5` |
| **Monthly Active Base** | DAU × 5 ratio | **34.5 Million MAU** | `DAU × 5` |

---

## 7. Self-Test Practice Problems & Interview Flashcards

### ✏️ Practice Problem 1:
> **Question**: *Your API handles 1,000 RPS. Assuming an average user makes 10 requests per day, what is your DAU capacity?*
>
> **Solution**:
> 1. `Daily Requests = 1,000 × 86,400 = 86,400,000 requests/day`.
> 2. `DAU = 86,400,000 ÷ 10 = 8.64 Million DAU`.

---

### 🃏 Flashcard 1: System Design Interview
> **Question**: *How do you estimate required RPS for an app targeting 10 Million Daily Active Users?*
>
> **Answer**:
> "If 10M users make an average of 20 requests per day, total daily volume is `200M requests/day`. Dividing by 86,400 seconds gives an average rate of `~2,315 RPS`. Applying a peak factor of `2.5×` for peak hours, the backend architecture must be provisioned to sustain `~5,787 Peak RPS`."
