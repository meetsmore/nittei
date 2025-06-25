use nittei::telemetry::init_subscriber;
use nittei_api::Application;
use nittei_infra::setup_context;
use nittei_utils::config;
use tikv_jemallocator::Jemalloc;
use tokio::signal;
use tracing::info;

/// Use jemalloc as the global allocator
/// This is a performance optimization for the application
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

/// Main wraps the `run` function with a tokio runtime (flavor can be decided via the `NITTEI__TOKIO_RUNTIME_FLAVOR` env var)
/// See crates/utils/src/config.rs for more details
fn main() {
    let runtime_flavor = nittei_utils::config::APP_CONFIG
        .tokio_runtime_flavor
        .as_str();
    match runtime_flavor {
        "current_thread" => {
            #[allow(clippy::unwrap_used)]
            let _ = tokio::runtime::Builder::new_current_thread()
                .enable_all() // Enable all features
                .build()
                .unwrap()
                .block_on(run());
        }
        "multi_thread" => {
            let mut runtime = tokio::runtime::Builder::new_multi_thread();

            // Enable all features
            let mut runtime = runtime.enable_all();

            // If the number of workers is set, use it
            // Otherwise, use the number of cores (Tokio default)
            if let Some(num_workers) =
                nittei_utils::config::APP_CONFIG.tokio_runtime_number_of_workers
            {
                let num_workers = num_workers.max(1); // Ensure at least 1 worker
                runtime = runtime.worker_threads(num_workers);
            }

            // Build the runtime and block on the run function
            #[allow(clippy::unwrap_used)]
            let _ = runtime.build().unwrap().block_on(run());
        }
        _ => {
            eprintln!("[start_runtime] Invalid tokio runtime flavor: {runtime_flavor}");
            std::process::exit(1);
        }
    }
}

/// The main function that will be run by the tokio runtime
async fn run() -> anyhow::Result<()> {
    // Initialize the subscriber for logging & tracing
    init_subscriber()?;

    print_runtime_info();

    let context = setup_context().await?;
    let (tx, rx) = tokio::sync::oneshot::channel::<()>();

    let app = Application::new(context).await?;

    // Listen for SIGINT (Ctrl+C) to shutdown the service
    // This sends a message on the channel to shutdown the server gracefully
    // It waits for a configurable amount of seconds (in order for the pod to be removed from the k8s service)
    // And then waits for the server to finish processing the current requests before shutting down
    tokio::spawn(async move {
        let ctrl_c = async {
            #[allow(clippy::expect_used)]
            signal::ctrl_c()
                .await
                .expect("failed to install signal handler (ctrl_c)");
        };

        let terminate = async {
            #[allow(clippy::expect_used)]
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler (terminate)")
                .recv()
                .await;
        };

        tokio::select! {
            _ = ctrl_c => info!("[main_shutdown_handler] Received SIGINT"),
            _ = terminate => info!("[main_shutdown_handler] Received SIGTERM"),
        }
        let _ = tx.send(());
    });

    // Start the application and block until it finishes
    app.start(rx).await?;

    info!("[main_shutdown_handler] shutdown complete");

    Ok(())
}

/// Print the runtime info
/// This is used to print the number of CPUs and the memory quota assigned to the container
fn print_runtime_info() {
    if !nittei_utils::config::APP_CONFIG.print_runtime_info {
        return;
    }

    let runtime = config::APP_CONFIG.tokio_runtime_flavor.as_str();
    tracing::info!("[print_runtime_info] Tokio runtime flavor: {runtime}");

    let number_of_cpus = num_cpus::get();
    tracing::info!("[print_runtime_info] Number of CPUs detected: {number_of_cpus}");

    // If the runtime is multi_thread, print the number of workers
    if runtime == "multi_thread" {
        let number_of_workers = match config::APP_CONFIG.tokio_runtime_number_of_workers {
            Some(number_of_workers) => number_of_workers,
            None => number_of_cpus,
        };
        tracing::info!("[print_runtime_info] Number of workers: {number_of_workers}");
    }

    if let Some(number_of_logical_cpus) = read_cpu_quota() {
        tracing::info!(
            "[print_runtime_info] Number of logical CPUs assigned to the container: {number_of_logical_cpus}"
        );
    }
    if let Some(memory_quota) = read_memory_quota() {
        tracing::info!(
            "[print_runtime_info] Memory quota assigned to the container: {memory_quota}"
        );
    }
}

/// Read the memory quota from the cgroup
/// This is used to limit the memory usage
/// If the quota is not set, return None
/// If the quota is set, return the memory quota in bytes
fn read_memory_quota() -> Option<f64> {
    use std::fs::read_to_string;
    let quota = read_to_string("/sys/fs/cgroup/memory.max").ok()?;
    let mut parts = quota.split_whitespace();
    let quota_bytes: f64 = parts.next()?.parse().ok()?;
    Some(quota_bytes)
}

/// Read the cpu quota from the cgroup
/// This is used to limit the number of workers
/// If the quota is not set, return None
/// If the quota is set, return the number of logical CPUs
///
/// Can be useful to know what is the max number of workers that can be used
fn read_cpu_quota() -> Option<f64> {
    use std::fs::read_to_string;

    let quota = read_to_string("/sys/fs/cgroup/cpu.max").ok()?;
    let mut parts = quota.split_whitespace();
    let quota_us: f64 = parts.next()?.parse().ok()?;
    let period_us: f64 = parts.next()?.parse().ok()?;
    Some(quota_us / period_us) // in logical CPUs
}
