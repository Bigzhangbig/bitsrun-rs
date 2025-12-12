I will fix the Windows Service issues, implement dual logging, and optimize the startup process as requested.

### 1. Fix Service Start Failure (Error 1053) & Optimize Initialization

The 1053 error occurs when the service doesn't report the `Running` status to the Service Control Manager (SCM) in a timely manner.

* **Move "Running" Status Update**: I will modify `src/windows_service.rs` to report `ServiceState::Running` *before* performing potentially slow initialization steps (like reading config or initializing the network runtime).

* **Error Handling**: If initialization fails after reporting `Running`, the service will gracefully stop and report the error.

### 2. Implement Dual Logging (Event Log + File)

I will replace the existing `winlog2` initialization with a custom `ServiceLogger` that writes to both:

* **Windows Event Log**: Using `windows-sys` API (`RegisterEventSource`, `ReportEvent`) to write logs to the system event viewer.

* **File Log**: Writing to `bitsrun_service.log` in the executable's directory.

* **Implementation**: I will create a new module `src/service_logger.rs` and update `Cargo.toml` to include necessary `windows-sys` features.

### 3. Improve SCM Interaction & Control

* **Stop Handling**: Ensure the service correctly transitions to `StopPending` and then `Stopped` upon receiving a stop signal.

* **Manual Run Support**: Modify `run_windows_service` to detect if the service dispatcher fails (which happens when running from console) and fall back to running the daemon in console mode. This satisfies the "Allow manual run" requirement.

### 4. Special Handling ("Skip auto-execution")

* I will verify and ensure that the service does not perform blocking operations (like user prompts) during startup.

* The fallback to console mode will allow manual execution without SCM interference.

### Plan Steps

1. **Update** **`Cargo.toml`**: Add `windows-sys` dependency with `Win32_System_EventLog`, `Win32_Foundation` features.
2. **Create** **`src/service_logger.rs`**: Implement the dual logging logic.
3. **Modify** **`src/windows_service.rs`**:

   * Integrate `ServiceLogger`.

   * Reorder `set_service_status` calls to fix Error 1053.

   * Add fallback logic for manual execution.

   * Ensure graceful shutdown on Stop signal.
4. **Verify**: I will verify the code changes by checking for compilation errors (using `cargo check` if possible) and ensuring logical correctness.

