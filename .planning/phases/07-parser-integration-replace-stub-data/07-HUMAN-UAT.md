---
status: partial
phase: 07-parser-integration-replace-stub-data
source: [07-VERIFICATION.md]
started: 2026-05-06T19:47:00Z
updated: 2026-05-06T19:47:00Z
---

## Current Test

[awaiting human testing]

## Tests

### 1. Real OT Volume Tempo Display
expected: Open the app with a real OT volume mounted. Navigate to a project detail view — tempo BPM should reflect the actual project tempo from the OT card (not 0 or a stub value).
result: [pending]

### 2. Slot Picker Shows Real Data
expected: Open a project Samples tab. Occupied slots show real filenames (e.g., kick_44100.wav) and empty slots show as empty — not 128 empty rows.
result: [pending]

### 3. Health Check Detects Missing Files
expected: Run health check on a project with a missing sample reference. Health tab shows an Error-severity item for the missing file.
result: [pending]

### 4. DETC-03 Suppression in Practice
expected: Health results do NOT flood with "assigned but not referenced by any track" Info messages for every occupied slot. Suppression guard is active.
result: [pending]

## Summary

total: 4
passed: 0
issues: 0
pending: 4
skipped: 0
blocked: 0

## Gaps
