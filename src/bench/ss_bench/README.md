State Store Benchmark (ss-bench)
===============

`ss_bench` is used to directly benchmark the state store of the system.

# Usage Example

We use a mock meta-service for `ss_bench`, and it may not be fully functional.

```shell
~/code/piestream/rust: cargo run --bin ss-bench -- \
 --benchmarks "writebatch,getseq,getrandom,prefixscanrandom,deleterandom" \
 --batch-size 1000 \
 --writes 10000 \
 --reads 500 \
 --scans 200 \
 --deletes 2000 \
 --concurrency-num 4 \
 --seed 233 \
 --statistics
```

# Parameters

## State Store

### Backend Types  (`--store`)

- `In-memory`
  
  - Format: `in-memory`(or `in_memory`)
  - Default

- `Hummock+MinIO`
  
  - Format: `hummock+minio://key:secret@address:port/bucket`
  - Example: `hummock+minio://hummockadmin:hummockadmin@127.0.0.1:9301/hummock001`

- `Hummock+S3`
  
  - Format: `hummock+s3://bucket`
  - Example: `hummock+s3://s3-ut`
  - Notice: some environment variables are required to be set
    - `AWS_REGION`
    - `AWS_ACCESS_KEY_ID`
    - `AWS_SECRET_ACCESS_KEY`

- `TiKV`
  
  - Format: `tikv://pd_address:port`
  - Example: `tikv://127.0.0.1:2379`

- `RocksDB`
  
  - Format: TBD

### Hummock Configurations

- `--table-size-mb`
  
  - Size (MB) of an SSTable
  - Default: 256

- `--block-size-kb`
  
  - Size (KB) of a block in an SSTable
  - Default: 64

- `--block-cache-capacity-mb`
  
  - Capacity of block cache
  - Default: 256

- `--meta-cache-capacity`
  
  - Capacity of meta cache
  - Default: 64

- `--shared-buffer-threshold-mb`
  
  - Threshold (MB) of shared buffer
  - Default: 192

- `--shared-buffer-capacity-mb`
  
  - Capacity (MB) of shared buffer
  - Default: 256

- `--shared-buffers-sync-parallelism`
  
  - Sync Parallelism of shared buffers
  - Default: 2

- `--bloom-false-positive`
  
  - Bloom Filter false positive rate
  - Default: 0.1

- `--compact-level-after-write`
  
  - 0 represent do nothing because all files will be synced to L0
  - Default: 0

- `--async-checkpoint-disabled`
  
  - Disable async checkpoint
  - Default: false

- `--write-conflict-detection-enabled`
  
  - Enable write conflict detection
  - Default: false

## Operations

### Concurrency Number (`--concurrency-num`)

- Concurrency number of each operation. Workloads of each concurrency are almost the same.
- Default: 1

### Operation Types (`--benchmarks`)

Comma-separated list of operations to run in the specified order. Following operations are supported:

- `writebatch`: write N key/values in sequential key order in async mode.
- `deleterandom`: delete N keys in random order. May delete a key/value many times even it has been deleted before during this operation. If the state store is already completely empty before this operation, randomly-generated keys would be deleted.
- `getrandom`: read N keys in random order. May read a key/value many times even it has been read before during this operation. If the state store is already completely empty before this operation, randomly-generated keys would be read instead.
- `getseq`: read N times sequentially. Panic if keys in the state store are less than number to get. But if the state store is completely empty, sequentially-generated keys would be read.
- `prefixscanrandom`: prefix scan N times in random order. May scan a prefix many times even it has been scanned before during this operation. If the state store is already completely empty before this operation, randomly-generated prefixes would be scanned in this empty state store.

Example: `--benchmarks "writebatch,prefixscanrandom,getrandom"`

### Operation Numbers

- `--num`

  - Number of key/values to place in database.
  - Default: 1000000

- `--deletes`

  - Number of deleted keys. If negative, do `--num` deletions.
  - Default: -1

- `--reads`

  - Number of read keys. If negative, do `--num` reads.
  - Default: -1

- `--scans`

  - Number of scanned prefixes. If negative, do `--num` scans.
  - Default: -1

- `--writes`

  - Number of written key/values. If negative, do `--num` writes.
  - Default: -1

- `--batch-size`

  - **Max** number of key/values in a batch. When the key/values are not evenly divided by the `--batch-size`, the last batch will be the remainder.
  - Default: 100

## Key/values Sizes

- `--key-size`
  
  - Size (bytes) of each user_key (non-prefix part of a key).
  - Default: 16

- `--key-prefix-size`
  
  - Size (bytes) of each prefix.
  - Default: 5

- `--keys-per-prefix`
  
  - Control **average** number of keys generated per prefix.
  - Default: 10

- `--value-size`
  
  - Size (bytes) of each value.
  - Default: 100

- `--seed`
  
  - Seed base for random number generators.
  - Default: 0

# Flag

- `--statistics`
  - Detailed statistics of storage backend

- `--calibrate-histogram`
  - Print performance by both self-measured metric and the state store metric system. This can be used to calibrate histogram parameters, especially bucket specification.

# Metrics

- Latency (`min/mean/P50/P95/P99/max/std_dev`)
- Throughput (`QPS/OPS/bytes_pre_second`)
