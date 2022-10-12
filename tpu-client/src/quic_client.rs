//! Simple client that connects to a given UDP port with the QUIC protocol and provides
//! an interface for sending transactions which is restricted by the server's flow control.

use {
    crate::{
        connection_cache_stats::ConnectionCacheStats,
        nonblocking::{
            quic_client::{
                QuicClient, QuicLazyInitializedEndpoint,
                QuicTpuConnection as NonblockingQuicTpuConnection,
            },
            tpu_connection::TpuConnection as NonblockingTpuConnection,
        },
        tpu_connection::TpuConnection,
    },
    lazy_static::lazy_static,
    solana_sdk::transport::Result as TransportResult,
    std::{
        net::SocketAddr,
        sync::{
            atomic::{AtomicBool, AtomicU64, Ordering},
            Arc,
        },
        thread::{sleep, Builder, JoinHandle},
        time::Duration,
    },
    tokio::runtime::Runtime,
};

lazy_static! {
    pub(crate) static ref RUNTIME: RuntimeWrapper = RuntimeWrapper::new();
}

pub(crate) struct RuntimeWrapper {
    pub(crate) runtime: Runtime,
    pub(crate) num_tasks: Arc<AtomicU64>,
    exit: Arc<AtomicBool>,
    sampling_thread: Option<JoinHandle<()>>,
}

impl RuntimeWrapper {
    fn sample_loop(exit: Arc<AtomicBool>, num_tasks: Arc<AtomicU64>) {
        while !exit.load(Ordering::Relaxed) {
            datapoint_warn!(
                "quic-runtime-stats",
                ("send_tasks", num_tasks.load(Ordering::Relaxed), i64)
            );
            let millis = Duration::from_millis(2);
            sleep(millis);
        }
    }
    pub fn new() -> Self {
        let num_tasks = Arc::new(AtomicU64::new(0));
        let exit = Arc::new(AtomicBool::new(false));

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .thread_name("quic-client")
            .enable_all()
            .build()
            .unwrap();

        let sampling_thread = {
            let exit_clone = exit.clone();
            let num_tasks_clone = num_tasks.clone();

            Some(
                Builder::new()
                    .name("quic-send-tasks-sampler".to_string())
                    .spawn(move || {
                        Self::sample_loop(exit_clone, num_tasks_clone);
                    })
                    .unwrap(),
            )
        };

        Self {
            runtime,
            num_tasks,
            exit,
            sampling_thread,
        }
    }
}

impl Drop for RuntimeWrapper {
    fn drop(&mut self) {
        self.exit.store(true, Ordering::Relaxed);
        self.sampling_thread
            .take()
            .unwrap()
            .join()
            .expect("quic send tasks reporting thread failed to join");
    }
}

pub struct QuicTpuConnection {
    inner: Arc<NonblockingQuicTpuConnection>,
}
impl QuicTpuConnection {
    pub fn new(
        endpoint: Arc<QuicLazyInitializedEndpoint>,
        tpu_addr: SocketAddr,
        connection_stats: Arc<ConnectionCacheStats>,
    ) -> Self {
        let inner = Arc::new(NonblockingQuicTpuConnection::new(
            endpoint,
            tpu_addr,
            connection_stats,
        ));
        Self { inner }
    }

    pub fn new_with_client(
        client: Arc<QuicClient>,
        connection_stats: Arc<ConnectionCacheStats>,
    ) -> Self {
        let inner = Arc::new(NonblockingQuicTpuConnection::new_with_client(
            client,
            connection_stats,
        ));
        Self { inner }
    }
}

impl TpuConnection for QuicTpuConnection {
    fn tpu_addr(&self) -> &SocketAddr {
        self.inner.tpu_addr()
    }

    fn send_wire_transaction_batch<T>(&self, buffers: &[T]) -> TransportResult<()>
    where
        T: AsRef<[u8]> + Send + Sync,
    {
        RUNTIME
            .runtime
            .block_on(self.inner.send_wire_transaction_batch(buffers))?;
        Ok(())
    }

    fn send_wire_transaction_async(&self, wire_transaction: Vec<u8>) -> TransportResult<()> {
        let inner = self.inner.clone();
        //drop and detach the task
        let _ = RUNTIME
            .runtime
            .spawn(async move { inner.send_wire_transaction(wire_transaction).await });
        Ok(())
    }

    fn send_wire_transaction_batch_async(&self, buffers: Vec<Vec<u8>>) -> TransportResult<()> {
        let inner = self.inner.clone();
        //drop and detach the task
        let _ = RUNTIME
            .runtime
            .spawn(async move { inner.send_wire_transaction_batch(&buffers).await });
        Ok(())
    }
}
