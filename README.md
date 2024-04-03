# kubo-exporter
A naive stats exporter for prometheus to gather stats from Kubo IPFS

This package just starts a simple and naive [Prometheus](https://prometheus.io/) exporter that returns stats gathered from [Kubo IPFS](https://github.com/ipfs/kubo). Some stats are missing, TLS support isn't enabled yet, and only the most trivial authentication is supported. It defaults to listening on TCP port 9200 (not registered with the Prometheus community), but can be changed.

The supported command line options are:

```
Usage: ipfs_exporter [OPTIONS] --secret <SECRET>

Options:
  -i, --ipfs-ip <IP>        IP address of IPFS node [default: 127.0.0.1]
  -o, --ipfs-port <PORT>    Port number of IPFS management API port [default: 5001]
  -l, --listen-ip <IP>      Listen address [default: 127.0.0.1]
  -p, --listen-port <PORT>  Listen port [default: 9200]
  -s, --secret <SECRET>     Authentication password. Probably better off passed in through a config file
  -h, --help                Print help
  -V, --version             Print version
```

The stats gathered look something like this:

```
# HELP kubo_ipfs_total_in_bytes Total number of bytes received
# TYPE kubo_ipfs_total_in_bytes counter
kubo_ipfs_total_in_bytes 98908244
# HELP kubo_ipfs_total_out_bytes Total number of bytes sent
# TYPE kubo_ipfs_total_out_bytes counter
kubo_ipfs_total_out_bytes 17012952
# HELP kubo_ipfs_rate_in_bytes Total rate incoming in bytes
# TYPE kubo_ipfs_rate_in_bytes gauge
kubo_ipfs_rate_in_bytes 229.6570947155488
# HELP kubo_ipfs_rate_out_bytes Total rate outgoing in bytes
# TYPE kubo_ipfs_rate_out_bytes gauge
kubo_ipfs_rate_out_bytes 1.6301735665456296
# HELP kubo_ipfs_repo_size_in_bytes Total capacity used by IPFS repository
# TYPE kubo_ipfs_repo_size_in_bytes counter
kubo_ipfs_repo_size_in_bytes 52396137
# HELP kubo_ipfs_repo_num_objects Number of objects currently persisted in IPFS repo
# TYPE kubo_ipfs_repo_num_objects gauge
kubo_ipfs_repo_num_objects 1723
# HELP kubo_ipfs_repo_storage_max_bytes Total number of bytes allowed in IPFS repository
# TYPE kubo_ipfs_repo_storage_max_bytes counter
kubo_ipfs_repo_storage_max_bytes 10000000000
# HELP kubo_ipfs_repo_path Path to Kubo IPFS repo
# TYPE kubo_ipfs_repo_path counter
kubo_ipfs_repo_path{path="/home/matthew/.ipfs"} 1
# HELP kubo_ipfs_repo_version IPFS repository version
# TYPE kubo_ipfs_repo_version counter
kubo_ipfs_repo_version{path="fs-repo@14"} 1
# HELP kubo_ipfs_bitswap_messages_received Total number of bitswap messages received
# TYPE kubo_ipfs_bitswap_messages_received counter
kubo_ipfs_bitswap_messages_received 2476
# HELP kubo_ipfs_bitswap_data_received Total number of bitswap bytes received
# TYPE kubo_ipfs_bitswap_data_received counter
kubo_ipfs_bitswap_data_received 0
# HELP kubo_ipfs_bitswap_data_sent Total number of bitswap bytes sent
# TYPE kubo_ipfs_bitswap_data_sent counter
kubo_ipfs_bitswap_data_sent 0
# HELP kubo_ipfs_bitswap_blocks_received Total number of bitswap bytes received
# TYPE kubo_ipfs_bitswap_blocks_received counter
kubo_ipfs_bitswap_blocks_received 0
# HELP kubo_ipfs_bitswap_blocks_sent Total number of bitswap bytes sent
# TYPE kubo_ipfs_bitswap_blocks_sent counter
kubo_ipfs_bitswap_blocks_sent 0
```
