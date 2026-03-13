# Code Review Report

## Summary

This document captures findings from a comprehensive code review of the k8s-mcp project.

## Critical Issues

### 1. Duplicate Tool Name (context.rs)
**Severity: High**

`ContextCurrentTool` and `ConfigurationViewTool` both define the tool name "configuration_view". This will cause one tool to overwrite the other in the registry.

**Location:** `src/tools/context.rs:84` and `src/tools/context.rs:155`

**Recommendation:** Rename `ContextCurrentTool`'s tool to something like "context_current".

### 2. Duplicate Function Definition (events.rs)
**Severity: Medium**

`format_events_table` is defined in both `src/tools/events.rs` and `src/format/table.rs`. The version in events.rs shadows the one in table.rs when used in the events tool.

**Location:** `src/tools/events.rs:78`

**Recommendation:** Remove the duplicate and use the function from the format module.

### 3. CPU Parsing Bug (metrics.rs)
**Severity: Medium**

In the `top_pods` method, CPU values from containers are incorrectly concatenated as strings instead of being summed:

```rust
let cpu = parse_cpu(&m.containers.iter()
    .map(|c| c.usage.get("cpu").map(|s| s.0.clone()).
    .collect::<String>());  // This joins strings, not values
```

**Location:** `src/k8s/metrics.rs:137-139`

**Recommendation:** Sum the parsed CPU values instead of concatenating strings.

## Security Issues

### 1. Secret Data Redaction
**Severity: Low**

The secret redaction in `get.rs` is good but could be more comprehensive. It only redacts the `data` field but not `stringData`.

**Location:** `src/tools/core/get.rs:82-92`

**Recommendation:** Also redact `stringData` field if present.

### 2. Kubeconfig Exposure
**Severity: Medium**

The `configuration_view` tool exposes full kubeconfig including credentials. While minified mode helps, users might accidentally expose sensitive data.

**Recommendation:** Add a warning in the tool description about credential exposure.

## Code Quality Issues

### 1. Missing Error Context
Several places use generic error messages without sufficient context for debugging.

### 2. Inconsistent Error Handling
Some tools use `?` directly while others wrap errors with context.

### 3. Unused Variables
In `metrics.rs`, the `container` variable is cloned but not used meaningfully.

## Style Issues

### 1. Inconsistent Naming
Tool names use different conventions:
- `resources_get`, `resources_list`, `resources_delete`
- `pods_get`, `pods_list`, `pods_delete`
- `nodes_top`, `pods_top`
- `configuration_contexts_list`, `configuration_view`

**Recommendation:** Standardize on a consistent naming convention.

### 2. Missing Documentation
Some public functions lack documentation comments.

## Performance Issues

### 1. Unnecessary Cloning
Several places clone when borrowing would suffice.

### 2. Repeated Discovery Calls
Discovery is cached but the check pattern is repeated in multiple places.

## Recommendations

1. Fix the duplicate tool name issue immediately
2. Fix the CPU parsing bug
3. Remove duplicate code
4. Add comprehensive error context
5. Standardize tool naming convention
6. Add security warnings to sensitive tools
7. Consider adding input validation for resource names