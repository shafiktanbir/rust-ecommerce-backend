# Post 032: The Cost-Based Optimizer (CBO) — Why PostgreSQL Ignores Your New Index
* **Target Audience**: Head of Data Infrastructure, Principal Engineers.
* **Viral Hook**: "You created an index, but PostgreSQL is still doing a Sequential Scan. Here is the math behind `random_page_cost` and table selectivity."
* **Core Problem**: When a query selects > 15-20% of rows in a table, the PostgreSQL Cost-Based Optimizer calculates that sequential disk reads are cheaper than random index page seeks (`random_page_cost = 4.0` vs `seq_page_cost = 1.0`).
* **Technical Math**:
  $$\text{Cost} = (\text{Page Fetches} \times \text{page_cost}) + (\text{Row Evaluated} \times \text{cpu_operator_cost})$$
* **Metrics Impact**: Tuned `random_page_cost = 1.1` for NVMe SSD storage, causing Pg Optimizer to correctly utilize indexes and reducing p95 catalog query latency by **84%**.
* **Cold Outreach DM**: "Hey [Name], saw [Company] is scaling database workloads on SSD/NVMe infrastructure. Default PostgreSQL settings still assume slow spinning hard drives (`random_page_cost = 4.0`), causing Pg to ignore valid indexes. Shared our NVMe tuning guide here!"

---
