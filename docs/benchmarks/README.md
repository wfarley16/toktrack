# Benchmark Results

Performance benchmarks for toktrack's JSONL parser.

## Latest (Phase 2 - rayon parallel)

| Metric | toktrack | ccusage | Speedup |
|--------|----------|---------|---------|
| Time (3.4GB, 2772 files) | ~1.0s | 40.3s | **~40x** |
| Throughput | ~3 GiB/s | ~73 MiB/s | - |

> Rust + simd-json + rayon parallel on Apple Silicon. Measured 2026-02-01.

## Dataset

- **Size**: 3.4 GB
- **Files**: 2,772 JSONL files
- **Location**: `~/.claude/projects/`

## Phase History

- [Phase 1: Baseline](./phase-1-baseline.md) - simd-json sequential parsing (4.7s / 628 MiB/s on 2.9GB)
- Phase 2: rayon parallel - file-level parallelism (~1.0s / ~3 GiB/s on 3.4GB)
