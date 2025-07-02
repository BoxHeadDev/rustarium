# ThreadPools

## 🧵 General Categories

### 1. Fixed Thread Pool
- Fixed number of threads that process tasks from a queue.
- Simple, predictable resource usage.
- Good for most web servers, job queues.

> Examples: `BasicThreadPool`, `MutexThreadPool`, Java's `Executors.newFixedThreadPool`

---

### 2. Cached / Elastic Thread Pool
- Spawns new threads as needed and reuses idle ones.
- Threads may time out and terminate after inactivity.
- Balances resource use and responsiveness.

> Example name: `ElasticThreadPool`, `CachedThreadPool`

---

### 3. Work-Stealing Thread Pool
- Each worker thread has its own queue.
- Idle workers "steal" work from others.
- Improves load balancing and CPU utilization.

> Real-world use: Rayon, Tokio, Java ForkJoinPool  
> Example name: `StealingThreadPool`, `ForkJoinPool`

---

### 4. Priority Thread Pool
- Tasks have priorities.
- Higher-priority jobs are executed before others.
- Requires priority queue.

> Example name: `PriorityThreadPool`

---

### 5. IO-Optimized (Event-driven) Pool
- Threads wait on async IO events, not blocked on CPU tasks.
- Not a traditional thread pool — more like an executor for IO.

> Example: Tokio runtime, async-std  
> Name idea: `AsyncThreadPool`, `IoExecutor`

---

### 6. CPU-bound vs IO-bound Pools
- Separate pools optimized for CPU-heavy and IO-heavy workloads.
- Avoids starving CPU-bound tasks with blocking IO.

> Example names: `CpuThreadPool`, `IoThreadPool`

---

## 🧪 Experimental or Advanced Models

### 7. Scoped Thread Pool
- Threads can spawn tasks that borrow local data.
- Useful in Rust for safety without `'static` requirements.

> Example: `rayon::scope`, `scoped_threadpool` crate

---

### 8. Single-threaded Executor
- Only one thread executes all tasks sequentially.
- Useful for testing, debugging, or predictable environments.

> Example name: `SingleThreadExecutor`

---

### 9. Thread-per-core Pool
- Spawns one thread per core (no oversubscription).
- May pin threads to cores using `core_affinity` or `num_cpus`.

> Example name: `CoreThreadPool`, `AffinityThreadPool`

---

### 10. Batching or Bulk Execution Pools
- Groups tasks into batches for execution.
- Useful in GPU workloads or bulk-parallel data processing.

> Example name: `BatchThreadPool`, `VectorizedExecutor`

---

## 📦 Crate-based Models (for inspiration)

| Crate         | Model                        |
|---------------|------------------------------|
| `rayon`       | Work stealing, data-parallel |
| `tokio`       | IO-focused, multilevel queues|
| `async-executor` | Lightweight, cooperative async |
| `smol`        | Tiny async runtime           |
| `threadpool`  | Simple fixed-size pool       |
| `crossbeam`   | Tools for lock-free concurrency |

---

## 💡 What to Explore Next?

| Idea                           | Why it’s valuable                                 |
|--------------------------------|---------------------------------------------------|
| `WorkStealingThreadPool`       | Learn task distribution, per-thread queues       |
| `ElasticThreadPool`            | Manage dynamic resources, thread TTL             |
| `CrossbeamPriorityThreadPool`  | Combine fast channels + prioritization           |
| `CoreThreadPool`               | Work with CPU affinity, `num_cpus`, performance  |
| `AsyncTaskPool`                | Bridge between sync thread pools and async world |

