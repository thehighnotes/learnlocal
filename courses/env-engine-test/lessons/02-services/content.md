# Services & Full Lifecycle

This lesson tests background services and the complete environment lifecycle.

## Background Services

Services are long-running processes started after setup commands. They
stay alive while student code runs and are killed after teardown.

Two readiness modes:
- **Delay mode**: waits `ready_delay_ms` then checks the process is alive.
- **Pattern mode**: reads stdout lines until one matches `ready_pattern`.

## Full Lifecycle

The complete execution order is:
1. Filesystem setup (dirs, files, symlinks)
2. Setup commands
3. Start services
4. Write student files
5. Run student code
6. Teardown commands
7. Kill services
8. State validation
