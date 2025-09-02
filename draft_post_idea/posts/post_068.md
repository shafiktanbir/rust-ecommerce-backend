# Post 068: Atomic CTEs (Common Table Expressions) — Multi-Table Updates in a Single Database Roundtrip
* **Target Audience**: Backend Developers, SQL Power Users.
* **Viral Hook**: "Why execute 3 separate database queries inside a transaction when 1 Atomic Data-Modifying CTE can do it all in 2 milliseconds?"
* **Core Problem**: Multiple network roundtrips between API workers and PostgreSQL (`BEGIN` $\rightarrow$ `UPDATE` $\rightarrow$ `INSERT` $\rightarrow$ `COMMIT`) add 10-15ms network latency per transaction.
* **Technical Query**:
  ```sql
  WITH decremented AS (
      UPDATE products 
      SET stock = stock - 1 
      WHERE id = $1 AND stock >= 1 
      RETURNING id, name, price
  )
  INSERT INTO orders (product_id, user_id, price)
  SELECT id, $2, price FROM decremented
  RETURNING id;
  ```
* **Metrics Impact**: Reduced transaction network roundtrips from 4 down to **1**; checkout latency cut from 18ms to **3.2ms**.
* **Cold Outreach DM**: "Hey [Name], saw your post on SQL performance tuning. Data-modifying CTEs allow executing inventory updates and order insertions in a single atomic SQL statement, reducing network roundtrips from 4 to 1. Documented our CTE checkout query here!"

---
