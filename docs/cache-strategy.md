# Cache Strategy

This document explains zet's approach to detecting file changes and maintaining cache consistency.

## Overview

Zet uses a three-tier detection system to efficiently identify when files need re-parsing while minimizing unnecessary work. The strategy is implemented in `src/collection.rs` and balances accuracy with performance.

## Three-Tier Detection

### Tier 1: Path Comparison
**Purpose:** Identify new, removed, and potentially changed files.

**Process:**
1. Scan collection directory for all `.md` files
2. Compare with paths stored in database
3. Partition files into:
   - **New**: Files on disk but not in database
   - **Intersecting**: Files present in both locations
   - **Removed**: Files in database but not on disk

**Outcome:** Provides the initial set of files to examine further.

### Tier 2: Timestamp Checking
**Purpose:** Identify files that may have been modified based on filesystem metadata.

**Process:**
1. For each intersecting file, read current `modified` timestamp
2. Compare with previously stored timestamp in database
3. Partition into:
   - **Modified**: Current timestamp differs from stored timestamp
   - **Unchanged**: Timestamps match

**Rationale:** Timestamps can change due to file moves, metadata updates, or other operations that don't affect content. This tier catches potential changes efficiently.

### Tier 3: Content Hashing
**Purpose:** Confirm actual content changes by comparing file content hashes.

**Process:**
1. For each file marked as modified by timestamp, read file content
2. Compute hash using `XxHash3_64` (fast, high-quality hash function)
3. Compare with previously stored hash in database
4. Partition into:
   - **Modified**: Hash has changed (content actually modified)
   - **Unchanged**: Hash matches (timestamp changed but content identical)

**Rationale:** This final step prevents unnecessary re-parsing when timestamps change but content doesn't (common with file operations, version control, etc.).

## Implementation Details

### Hash Function Choice
- **Algorithm**: `XxHash3_64` via `twox_hash` crate
- **Properties**: Fast computation, good distribution, 64-bit output
- **Rationale**: Optimized for speed while maintaining low collision probability for typical document sizes

### Storage Requirements
The database stores for each document:
- File path (for tier 1 comparison)
- Last modified timestamp (for tier 2 comparison)
- Content hash (for tier 3 comparison)
- Document UUID (for relationship tracking)

### Performance Characteristics

**Tier 1 (Path Comparison):**
- **Cost**: O(n) filesystem scan + O(n) database query
- **Benefit**: Identifies structural changes (new/removed files)

**Tier 2 (Timestamp Checking):**
- **Cost**: O(m) filesystem metadata reads where m = intersecting files
- **Benefit**: Fast filtering of potentially changed files

**Tier 3 (Content Hashing):**
- **Cost**: O(k) file content reads + hash computation where k = timestamp-modified files
- **Benefit**: Eliminates false positives from timestamp-only changes

## Cache Invalidation Scenarios

### New File Added
- **Detection**: Tier 1 (path comparison)
- **Action**: Parse and add to database

### File Content Modified
- **Detection**: Progresses through all three tiers
- **Action**: Re-parse AST, update links, refresh cache entry

### File Moved/Renamed
- **Detection**: Tier 1 shows old path removed, new path added
- **Action**: Remove old entry, parse new location
- **Note**: Link relationships may need updating

### File Deleted
- **Detection**: Tier 1 (path in database but not on disk)
- **Action**: Remove from database, update dependent link relationships

### Timestamp-Only Changes
- **Detection**: Tier 2 shows modification, Tier 3 shows unchanged content
- **Action**: Update stored timestamp, no re-parsing needed

### External File Operations
- **Examples**: Git operations, file sync, backup restoration
- **Handling**: System gracefully handles bulk timestamp changes through tier 3 filtering

## Advantages

1. **Efficiency**: Avoids re-parsing files that haven't actually changed
2. **Accuracy**: Content hashing eliminates false positives from metadata changes
3. **Scalability**: Early filtering reduces work for large collections
4. **Robustness**: Handles various file operation scenarios gracefully

## Trade-offs

1. **Storage Overhead**: Must store timestamps and hashes for each file
2. **Hash Computation**: CPU cost for reading and hashing modified files
3. **Complexity**: Three-tier system more complex than simple timestamp checking

The strategy prioritizes correctness and efficiency, making it suitable for both interactive use and automated workflows where file operations are common.