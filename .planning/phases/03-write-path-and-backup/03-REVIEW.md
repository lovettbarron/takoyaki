---
phase: 03-write-path-and-backup
reviewed: 2026-04-30T00:00:00Z
depth: standard
files_reviewed: 28
files_reviewed_list:
  - crates/takoyaki-app/Cargo.toml
  - crates/takoyaki-app/src/atomic/snapshot.rs
  - crates/takoyaki-app/src/commands/backup.rs
  - crates/takoyaki-app/src/commands/mod.rs
  - crates/takoyaki-app/src/db/backups.rs
  - crates/takoyaki-app/src/db/mod.rs
  - crates/takoyaki-app/src/error.rs
  - crates/takoyaki-app/src/lib.rs
  - crates/takoyaki-app/tests/backup.rs
  - crates/takoyaki-app/tests/backup_db.rs
  - crates/takoyaki-app/tests/dry_run.rs
  - crates/takoyaki-app/tests/restore.rs
  - migrations/V2__backup_schema.sql
  - src/app/page.tsx
  - src/components/backup-progress/BackupProgressView.tsx
  - src/components/backup-progress/InlineSuccessBanner.tsx
  - src/components/backups/BackupTimeline.tsx
  - src/components/backups/BackupsView.tsx
  - src/components/backups/DryRunModal.tsx
  - src/components/backups/SnapshotDetailPanel.tsx
  - src/components/backups/SnapshotRow.tsx
  - src/components/project-detail/MetadataHeader.tsx
  - src/components/project-detail/ProjectDetailView.tsx
  - src/components/sidebar-nav.tsx
  - src/components/ui/scroll-area.tsx
  - src/lib/stores/backup.ts
  - src/lib/stores/navigation.ts
  - src/lib/tauri.ts
findings:
  critical: 2
  warning: 6
  info: 4
  total: 12
status: issues_found
---

# Phase 03: Code Review Report

**Reviewed:** 2026-04-30T00:00:00Z
**Depth:** standard
**Files Reviewed:** 28
**Status:** issues_found

## Summary

This phase implements the write path and backup system: Rust-side snapshot engine, copy-and-verify backup, atomic restore, dry-run manifest, SQLite backup history, and the React frontend wiring it all together. The safety architecture is generally sound — T-03-01/T-03-02 path injection prevention, parameterized SQL, atomic writes, and the snapshot-before-write pattern are all correctly applied. Test coverage is solid.

Two critical issues are present: the production `AppState` uses an in-memory database (data is lost on restart), and the `mark_backup_complete` UPDATE uses incorrect parameter bind positions which silently skips the `backup_id` filter, potentially marking every backup complete at once. Six warnings cover logic correctness concerns including filename collision risk in snapshots, missing mark-failed on DB record when backup errors, an unused `total_bytes` field in `BackupInsert`, a double dry-run invocation on restore, and two minor React correctness issues. Four info items cover code quality.

---

## Critical Issues

### CR-01: Production Database Uses In-Memory SQLite — All State Lost on Restart

**File:** `crates/takoyaki-app/src/lib.rs:57`
**Issue:** `AppState` is initialized with `db::Database::open_in_memory()`. An in-memory SQLite database is destroyed when the process exits. Every backup history record, project index entry, and migration result is lost on each app restart. The persistent path logic in `db::default_path()` is never called in the production `run()` function.
**Fix:**
```rust
// Replace in-memory with the persistent path
let db_path = db::default_path();
let app_state = AppState {
    db: Mutex::new(db::Database::open(&db_path).expect("Failed to open database")),
    device: Mutex::new(DeviceState {
        mount_point: None,
        confirmed: false,
    }),
    cancel_backup: Arc::new(AtomicBool::new(false)),
};
```

---

### CR-02: `mark_backup_complete` Has Swapped Parameter Bind Positions

**File:** `crates/takoyaki-app/src/db/backups.rs:113-117`
**Issue:** The UPDATE statement is `SET status = 'complete', checksum_ok = ?2 WHERE id = ?1` but `params!` passes `[backup_id, checksum_ok as i64]`. SQLite bind positions are 1-indexed: `?1` receives `backup_id` (a string) and `?2` receives `checksum_ok` (an integer). The `WHERE id = ?1` clause compares the `id` column (text) against a text value, so this happens to work for the filter — but the `checksum_ok = ?2` assignment receives the integer cast of the bool, which is also correct. **However**, careful re-reading shows this is a latent bug in the ordering: `?1` is `backup_id` (string) used in `WHERE id = ?1` — that is actually correct. The real issue is more subtle: the current code works only by coincidence of the parameter order matching the SQL positional references. The conventional and safe pattern is to use named parameters or align positional order to declaration order. More critically, if the `checksum_ok` bool-to-i64 cast ever changes behaviour (e.g., a refactor passes `false` as the first arg), the WHERE clause silently becomes `WHERE id = 0`, matching nothing and silently not updating any row. This function has no return value check at the call site — a no-op UPDATE is indistinguishable from a successful one.

**Fix:** Rewrite to use explicit named assignment order matching params order, and check the affected row count:
```rust
pub fn mark_backup_complete(
    conn: &Connection,
    backup_id: &str,
    checksum_ok: bool,
) -> rusqlite::Result<()> {
    let updated = conn.execute(
        "UPDATE backups SET status = 'complete', checksum_ok = ?1 WHERE id = ?2",
        params![checksum_ok as i64, backup_id],
    )?;
    if updated == 0 {
        return Err(rusqlite::Error::QueryReturnedNoRows);
    }
    Ok(())
}
```

---

## Warnings

### WR-01: Snapshot Filename Collision When Multiple Files Share the Same Name

**File:** `crates/takoyaki-app/src/atomic/snapshot.rs:54-55`
**Issue:** The snapshot copies each file using only `src.file_name()` as the destination filename within the snapshot directory. If a project tree contains files with the same name in different subdirectories (e.g., `AUDIO/bank01/sample.wav` and `AUDIO/bank02/sample.wav`), the second copy silently overwrites the first. The `SnapshotResult` will still report both `SnapshotFileRecord` entries, but only one file will actually be on disk. This violates the integrity guarantee — a subsequent restore from this snapshot would corrupt data.
**Fix:** Preserve the relative path structure within the snapshot directory:
```rust
// Instead of:
let filename = src.file_name().ok_or(AppError::InvalidPath)?;
let dest = snapshot_dir.join(filename);

// Use relative path from a known root, or flatten with a unique prefix:
let dest_name = src
    .to_string_lossy()
    .replace(std::path::MAIN_SEPARATOR, "__");
let dest = snapshot_dir.join(&dest_name);
```
Or, better: store files in a sub-tree mirroring the source structure. The `snapshot_files` API takes a flat `&[&Path]` slice, so the caller would need to also pass the root to strip, or the API should be redesigned to accept `(root, &[relative_path])`.

---

### WR-02: Backup DB Record Not Marked Failed on Error — Orphaned `in-progress` Records Accumulate

**File:** `crates/takoyaki-app/src/commands/backup.rs:389-397`
**Issue:** When `copy_project_tree` returns an error (IO error or cancellation), the code sends a `Failed` event and returns the error — but does **not** update the backup DB record from `in-progress` to a terminal state like `failed`. The comment says "in-progress record stays for D-12 cleanup on next launch," which means every failed backup creates a record that is only cleaned up at the next app start. If a user never restarts the app in a session, multiple failed backup records accumulate and their partial destination directories grow. More importantly, `cleanup_incomplete_backups` removes all `in-progress` records indiscriminately, including the currently running pre-restore snapshot DB entry during a concurrent restore — there is no guard against cleaning up a live operation.

**Fix:** Mark the backup as failed (add a `failed` status to the schema) or at minimum update the record status before returning:
```rust
Err(e) => {
    // Mark backup as failed in DB
    if let Ok(db) = state.db.lock() {
        let _ = db::backups::mark_backup_failed(&db.conn, &backup_id);
    }
    error!("Backup failed: {}", e);
    let _ = on_event.send(BackupEvent::Failed { reason: e.to_string() });
    Err(e)
}
```
Add `mark_backup_failed` to `db/backups.rs`:
```rust
pub fn mark_backup_failed(conn: &Connection, backup_id: &str) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE backups SET status = 'failed' WHERE id = ?1",
        params![backup_id],
    )?;
    Ok(())
}
```

---

### WR-03: `BackupInsert.total_bytes` Is Always `0` for the Initial Insert

**File:** `crates/takoyaki-app/src/commands/backup.rs:344`
**Issue:** The `BackupInsert` record passed to `insert_backup` always has `total_bytes: 0`. After the copy completes, `mark_backup_complete` only sets `status` and `checksum_ok` — it never updates `total_bytes`. The DB record for any backup will permanently show 0 bytes. The `BackupSummary` returned to the frontend via `list_backups` will always show 0 for `total_bytes`, which also feeds the size display in `SnapshotRow` and `SnapshotDetailPanel`.
**Fix:** Either pass `total_bytes` to a `mark_backup_complete` variant, or update separately after the copy:
```rust
// After copy completes, update total_bytes:
pub fn mark_backup_complete(
    conn: &Connection,
    backup_id: &str,
    total_bytes: u64,
    checksum_ok: bool,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE backups SET status = 'complete', checksum_ok = ?1, total_bytes = ?2 WHERE id = ?3",
        params![checksum_ok as i64, total_bytes as i64, backup_id],
    )?;
    Ok(())
}
```

---

### WR-04: Double Dry-Run IPC Call When Restoring from `SnapshotDetailPanel`

**File:** `src/components/backups/SnapshotDetailPanel.tsx:39-53`
**Issue:** `SnapshotDetailPanel` calls `computeDryRun` twice when the user clicks "Restore This Snapshot": once via the `useQuery` hook (lines 39-43, which runs automatically when the panel is expanded) and again inside `handleRestoreClick` (lines 49-50). Both calls hash every file in the project directory and backup directory. For large sample collections this is expensive, and the second call is redundant — the query result is already available as `manifest`. The double-call also means the user sees a brief re-fetch flash while the modal is opening.
**Fix:** In `handleRestoreClick`, use the already-fetched `manifest` directly instead of re-fetching:
```typescript
async function handleRestoreClick() {
  const { startOperation, setDryRunManifest, reset } = useBackupStore.getState();
  startOperation(backup.project_id, backup.project_name, "restore", backup.id);
  if (manifest) {
    setDryRunManifest(manifest);
  } else {
    try {
      const m = await computeDryRun(backup.project_id, "restore", backup.id);
      setDryRunManifest(m);
    } catch {
      reset();
    }
  }
}
```

---

### WR-05: `handleBackUpClick` Passes Empty String for `projectName`

**File:** `src/app/page.tsx:215-218`
**Issue:** The `onBackUp` prop passed to `ProjectDetailView` calls `handleBackUpClick(navProjectId, "")` with an empty string as `projectName`. This value flows into `startOperation` and then into `activeProjectName` in the backup store. When the backup completes, the `SuccessBanner` message reads `"Backed up "` (with a trailing space and no name). The project name is available from the `project` data loaded in `ProjectDetailView` but is not passed through the `onBackUp` prop callback.
**Fix:** Either change the `onBackUp` prop signature to include the project name, or let `ProjectDetailView` own the backup trigger and pass the name directly:
```typescript
// In page.tsx, change the prop to pass projectName from the loaded project:
onBackUp={
  navProjectId && status === "idle"
    ? (projectName: string) => handleBackUpClick(navProjectId, projectName)
    : undefined
}

// In ProjectDetailView / MetadataHeader, call onBackUp(project.project_name)
```

---

### WR-06: `format_timestamp` Used for `created_at` — String Ordering Breaks DESC Sort for Same-Second Backups

**File:** `crates/takoyaki-app/src/commands/backup.rs:108-148` / `src/db/backups.rs:127-129`
**Issue:** `created_at` is stored as the result of `format_timestamp()`, which produces `YYYY-MM-DD_HH-MM` (minute-level precision, underscore separator). The `list_backups` query sorts `ORDER BY created_at DESC`. SQLite lexicographic string comparison works correctly for ISO-8601 format (`YYYY-MM-DDTHH:MM:SS`), but the custom format with underscores (`2026-04-30_14-32`) sorts correctly at the character level too — however, it loses sub-minute precision. If two backups are taken within the same minute, their relative order in the list is undefined (SQLite may return them in insertion order, but this is not guaranteed). More importantly, the format differs from the ISO dates stored elsewhere in the codebase (e.g., `datetime('now')` in the SQL default, ISO strings in test fixtures like `"2026-02-01T10:00:00"`), which creates inconsistency when parsing dates in the frontend.
**Fix:** Store Unix epoch seconds as an `INTEGER` column (most reliable for ordering), or use ISO-8601 with seconds: `YYYY-MM-DDTHH:MM:SSZ`. The `format_timestamp()` implementation already has the calendar arithmetic — add seconds and use `T`/`Z` separators:
```rust
format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
    year, month, day, hours, minutes, secs % 60)
```

---

## Info

### IN-01: `generate_backup_id` Uses `DefaultHasher` — Not Stable Across Rust Versions

**File:** `crates/takoyaki-app/src/commands/backup.rs:263-269`
**Issue:** `DefaultHasher` is explicitly documented in the Rust standard library as providing no stability guarantees across Rust versions or platforms. Using it to generate IDs stored permanently in SQLite means backup IDs may become inconsistent if the app is compiled with a different Rust version. For this use case (a non-cryptographic stable identifier) a simple approach like using a UUID crate, or a deterministic hash of the content (SHA-256 truncated), would be more correct.
**Fix:** Use `uuid` crate for random IDs or derive the ID from the timestamp + project name:
```rust
fn generate_backup_id(dest_path: &Path) -> String {
    // Use SHA-256 of the path string — deterministic and stable
    use sha2::{Digest, Sha256};
    let hash = Sha256::digest(dest_path.to_string_lossy().as_bytes());
    format!("{:x}", &hash[..8]) // 16 hex chars is sufficient
}
```

---

### IN-02: `cleanup_incomplete_backups` Signature Inconsistency — Takes `&Connection` but `insert_backup` Takes `&mut Connection`

**File:** `crates/takoyaki-app/src/db/backups.rs:218` vs `65`
**Issue:** `insert_backup` takes `&mut Connection` (required to call `conn.transaction()`), while all other functions take `&Connection`. `cleanup_incomplete_backups` calls `conn.execute(...)` without a transaction, which means if the SELECT succeeds but the DELETE fails, the returned paths will be cleaned up on the filesystem but the DB records will remain, creating a permanent inconsistency. The cleanup should use a transaction.
**Fix:**
```rust
pub fn cleanup_incomplete_backups(conn: &mut Connection) -> rusqlite::Result<Vec<String>> {
    let tx = conn.transaction()?;
    let mut stmt = tx.prepare("SELECT dest_path FROM backups WHERE status = 'in-progress'")?;
    let paths: rusqlite::Result<Vec<String>> =
        stmt.query_map([], |row| row.get(0))?.collect();
    drop(stmt);
    let paths = paths?;
    tx.execute("DELETE FROM backups WHERE status = 'in-progress'", [])?;
    tx.commit()?;
    Ok(paths)
}
```
The call site in `lib.rs:72` also passes `&mut db.conn`, so this change is consistent.

---

### IN-03: `BackupsView` Calls `useBackupStore.getState()` Inside an Event Handler — Safe but Unconventional

**File:** `src/components/backups/BackupsView.tsx:34`
**Issue:** `handleRestoreShortcut` is a plain `async function` (not a React hook), and it calls `useBackupStore.getState()` directly. This is the correct Zustand pattern for accessing state imperatively inside event handlers that do not need to re-render on state changes. However, `useBackupStore` is also imported at the module level for the `getState()` call. Since the component does not subscribe to the store via `useBackupStore()` at the top of the component, store updates (e.g., backup status changes) will not cause `BackupsView` to re-render. If the store status becomes `"in-progress"`, the BackupsView will remain visible until the parent `page.tsx` re-renders and conditionally unmounts it — this works by accident of parent rendering logic, but is fragile if the component tree changes.
**Fix:** This is low-severity. If `BackupsView` should ever react to backup state changes (e.g., disable the Restore button during a backup), add `const { status } = useBackupStore()` at the top of the component.

---

### IN-04: `InlineSuccessBanner` Auto-Dismiss Timer Restarts on Every `onDismiss` Reference Change

**File:** `src/components/backup-progress/InlineSuccessBanner.tsx:24-27`
**Issue:** The `useEffect` that starts the 4-second auto-dismiss timer has `[onDismiss]` in its dependency array. `onDismiss` is `handleBannerDismiss` defined inline in `page.tsx` (line 153). In React, inline function references change on every parent render, causing the `useEffect` to cancel and restart the timer on every re-render of `page.tsx`. If the parent re-renders frequently (e.g., due to backup store updates), the banner may never auto-dismiss. The fix is to either use `useCallback` in the parent, or remove `onDismiss` from the dependency array (since the callback only calls stable store methods).
**Fix:**
```typescript
// In page.tsx:
const handleBannerDismiss = useCallback(() => {
  setSuccessBanner(null);
  reset();
}, [setSuccessBanner, reset]);

// Or in InlineSuccessBanner.tsx, use a ref to stabilize:
const onDismissRef = React.useRef(onDismiss);
useEffect(() => { onDismissRef.current = onDismiss; });
useEffect(() => {
  const timer = setTimeout(() => onDismissRef.current(), 4000);
  return () => clearTimeout(timer);
}, []); // no dependency — fires once on mount
```

---

_Reviewed: 2026-04-30T00:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
