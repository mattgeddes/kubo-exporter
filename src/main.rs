/// A simple prometheus exporter wrapper around the Kubo stats API for IPFS.
/// Currently:
///   - (Barely) supports Authentication
///   - Doesn't support TLS
///   - Doesn't do a great job at handling errors
///   - Generally works.
use anyhow::Result;
use clap::Parser;
use log::debug;
use prometheus_exporter_base::prelude::*;
use reqwest::Client;
use serde_derive::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio;

#[derive(Debug, Parser, Clone)]
#[clap(author, version, about)]
struct CliOpts {
    /// IP address of IPFS node
    #[clap(
        short = 'i',
        long,
        value_parser,
        value_name = "IP",
        default_value = "127.0.0.1"
    )]
    ipfs_ip: String,
    /// Port number of IPFS management API port
    #[clap(
        short = 'o',
        long,
        value_parser,
        value_name = "PORT",
        default_value = "5001"
    )]
    ipfs_port: u32,
    /// Listen address
    #[clap(
        short = 'l',
        long,
        value_parser,
        value_name = "IP",
        default_value = "127.0.0.1"
    )]
    listen_ip: String,
    /// Listen port
    #[clap(
        short = 'p',
        long,
        value_parser,
        value_name = "PORT",
        default_value = "9200"
    )]
    listen_port: u32,
    /// Authentication password. Probably better off passed in through a config file.
    #[clap(short = 's', long, value_parser, value_name = "SECRET")]
    secret: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct BandwidthStats {
    total_in: i64,
    total_out: i64,
    rate_in: f64,
    rate_out: f64,
}
const BW_URI: &str = "/api/v0/stats/bw";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RepoStats {
    repo_size: f64,
    num_objects: u64,
    storage_max: f64,
    repo_path: String,
    version: String,
}
const REPO_URI: &str = "/api/v0/stats/repo";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct BitswapStats {
    blocks_received: u64,
    blocks_sent: u64,
    data_received: u64,
    data_sent: u64,
    dup_blks_received: u64,
    dup_data_received: u64,
    messages_received: u64,
    peers: Vec<String>,
    provide_buf_len: i32,
}
const BSWAP_URI: &str = "/api/v0/stats/bitswap";

async fn gather_metrics(options: &CliOpts) -> Result<String> {
    let bw = gather_bw_metrics(options).await?;
    let repo = gather_repo_metrics(options).await?;
    let bswap = gather_bswap_metrics(options).await?;

    let ret = format!("{}{}{}", bw, repo, bswap);
    Ok(ret)
}

// Gather repo metrics
async fn gather_repo_metrics(options: &CliOpts) -> Result<String> {
    // Create a new HTTP client.
    let client = Client::new();
    let url = format!(
        "http://{}:{}{}",
        options.ipfs_ip, options.ipfs_port, REPO_URI
    );
    let response = client.post(url).send().await?;

    debug!("Repo stats call result: {}", response.status());
    let body = response.text().await?;
    debug!("Body: {}", body);
    let repo_stats: RepoStats = serde_json::from_str(&body)?;
    debug!("Repo: {:?}", repo_stats);

    // Capacity used
    let used = PrometheusMetric::build()
        .with_name("kubo_ipfs_repo_size_in_bytes")
        .with_metric_type(MetricType::Counter)
        .with_help("Total capacity used by IPFS repository")
        .build()
        .render_and_append_instance(&PrometheusInstance::new().with_value(repo_stats.repo_size))
        .render();

    // Number of objects
    let num_objects = PrometheusMetric::build()
        .with_name("kubo_ipfs_repo_num_objects")
        .with_metric_type(MetricType::Gauge)
        .with_help("Number of objects currently persisted in IPFS repo")
        .build()
        .render_and_append_instance(&PrometheusInstance::new().with_value(repo_stats.num_objects))
        .render();

    // Max capacity
    let total = PrometheusMetric::build()
        .with_name("kubo_ipfs_repo_storage_max_bytes")
        .with_metric_type(MetricType::Counter)
        .with_help("Total number of bytes allowed in IPFS repository")
        .build()
        .render_and_append_instance(&PrometheusInstance::new().with_value(repo_stats.storage_max))
        .render();

    // path to repo
    let path = PrometheusMetric::build()
        .with_name("kubo_ipfs_repo_path")
        .with_metric_type(MetricType::Counter)
        .with_help("Path to Kubo IPFS repo")
        .build()
        .render_and_append_instance(
            &PrometheusInstance::new()
                .with_label("path", repo_stats.repo_path.clone().as_str())
                .with_value(1),
        )
        .render();

    // path to repo
    let version = PrometheusMetric::build()
        .with_name("kubo_ipfs_repo_version")
        .with_metric_type(MetricType::Counter)
        .with_help("IPFS repository version")
        .build()
        .render_and_append_instance(
            &PrometheusInstance::new()
                .with_label("path", repo_stats.version.clone().as_str())
                .with_value(1),
        )
        .render();

    let ret = format!("{}{}{}{}{}", used, num_objects, total, path, version);
    Ok(ret)
}

// Gather bandwidth metrics
async fn gather_bw_metrics(options: &CliOpts) -> Result<String> {
    // Create a new HTTP client.
    let client = Client::new();
    let url = format!("http://{}:{}{}", options.ipfs_ip, options.ipfs_port, BW_URI);
    let response = client.post(url).send().await?;

    debug!("Bandwidth stats call result: {}", response.status());
    let body = response.text().await?;
    debug!("Body: {}", body);
    let bw_stats: BandwidthStats = serde_json::from_str(&body)?;
    debug!("Bandwidth: {:?}", bw_stats);

    // total in metric
    let tin = PrometheusMetric::build()
        .with_name("kubo_ipfs_total_in_bytes")
        .with_metric_type(MetricType::Counter)
        .with_help("Total number of bytes received")
        .build()
        .render_and_append_instance(&PrometheusInstance::new().with_value(bw_stats.total_in))
        .render();

    // total out metric
    let tout = PrometheusMetric::build()
        .with_name("kubo_ipfs_total_out_bytes")
        .with_metric_type(MetricType::Counter)
        .with_help("Total number of bytes sent")
        .build()
        .render_and_append_instance(&PrometheusInstance::new().with_value(bw_stats.total_out))
        .render();

    // rate in metric
    let rin = PrometheusMetric::build()
        .with_name("kubo_ipfs_rate_in_bytes")
        .with_metric_type(MetricType::Gauge)
        .with_help("Total rate incoming in bytes")
        .build()
        .render_and_append_instance(&PrometheusInstance::new().with_value(bw_stats.rate_in))
        .render();

    // rate out metric
    let rout = PrometheusMetric::build()
        .with_name("kubo_ipfs_rate_out_bytes")
        .with_metric_type(MetricType::Gauge)
        .with_help("Total rate outgoing in bytes")
        .build()
        .render_and_append_instance(&PrometheusInstance::new().with_value(bw_stats.rate_out))
        .render();

    let ret = format!("{}{}{}{}", tin, tout, rin, rout);
    Ok(ret)
}

// Gather bitswap metrics
async fn gather_bswap_metrics(options: &CliOpts) -> Result<String> {
    // Create a new HTTP client.
    let client = Client::new();
    let url = format!(
        "http://{}:{}{}",
        options.ipfs_ip, options.ipfs_port, BSWAP_URI
    );
    let response = client.post(url).send().await?;

    debug!("Bitswap stats call result: {}", response.status());
    let body = response.text().await?;
    debug!("Body: {}", body);
    let bs_stats: BitswapStats = serde_json::from_str(&body)?;
    debug!("Bitswap: {:?}", bs_stats);

    // bitswap messages
    let msgs_rcvd = PrometheusMetric::build()
        .with_name("kubo_ipfs_bitswap_messages_received")
        .with_metric_type(MetricType::Counter)
        .with_help("Total number of bitswap messages received")
        .build()
        .render_and_append_instance(
            &PrometheusInstance::new().with_value(bs_stats.messages_received),
        )
        .render();

    // data received
    let data_rcvd = PrometheusMetric::build()
        .with_name("kubo_ipfs_bitswap_data_received")
        .with_metric_type(MetricType::Counter)
        .with_help("Total number of bitswap bytes received")
        .build()
        .render_and_append_instance(&PrometheusInstance::new().with_value(bs_stats.data_received))
        .render();

    // data sent
    let data_sent = PrometheusMetric::build()
        .with_name("kubo_ipfs_bitswap_data_sent")
        .with_metric_type(MetricType::Counter)
        .with_help("Total number of bitswap bytes sent")
        .build()
        .render_and_append_instance(&PrometheusInstance::new().with_value(bs_stats.data_sent))
        .render();

    // blocks received
    let blocks_rcvd = PrometheusMetric::build()
        .with_name("kubo_ipfs_bitswap_blocks_received")
        .with_metric_type(MetricType::Counter)
        .with_help("Total number of bitswap bytes received")
        .build()
        .render_and_append_instance(&PrometheusInstance::new().with_value(bs_stats.blocks_received))
        .render();

    // blocks sent
    let blocks_sent = PrometheusMetric::build()
        .with_name("kubo_ipfs_bitswap_blocks_sent")
        .with_metric_type(MetricType::Counter)
        .with_help("Total number of bitswap bytes sent")
        .build()
        .render_and_append_instance(&PrometheusInstance::new().with_value(bs_stats.blocks_sent))
        .render();

    let ret = format!(
        "{}{}{}{}{}",
        msgs_rcvd, data_rcvd, data_sent, blocks_rcvd, blocks_sent
    );
    Ok(ret)
}

#[tokio::main]
async fn main() -> Result<()> {
    // parse command line arguments
    let cli = CliOpts::parse();

    env_logger::init(); // initialise logging framework

    let addr = format!("{}:{}", cli.listen_ip, cli.listen_port);
    let bind_addr: SocketAddr = addr.parse()?;
    debug!("Bind address: {}", bind_addr);

    let password = cli.secret.clone();

    let server_options = ServerOptions {
        addr: bind_addr,
        authorization: Authorization::Basic(password),
    };

    render_prometheus(
        server_options,
        cli.to_owned(),
        |_request, options| async move { Ok(gather_metrics(&options).await?) },
    )
    .await;

    Ok(())
}
