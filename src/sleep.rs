use futures_util::StreamExt;
use tokio::sync::mpsc;
use zbus::proxy;

#[proxy(
    interface = "org.freedesktop.login1.Manager",
    default_service = "org.freedesktop.login1",
    default_path = "/org/freedesktop/login1"
)]
trait Manager {
    #[zbus(signal)]
    fn prepare_for_sleep(&self, start: bool) -> zbus::Result<()>;
}

pub struct WakeWatcher {
    rx: Option<mpsc::Receiver<()>>,
}

impl WakeWatcher {
    pub async fn new() -> Self {
        match Self::setup().await {
            Ok(rx) => Self { rx: Some(rx) },
            Err(e) => {
                log::warn!("Sleep/wake watcher unavailable: {}", e);
                Self { rx: None }
            }
        }
    }

    async fn setup() -> Result<mpsc::Receiver<()>, Box<dyn std::error::Error>> {
        let conn = zbus::Connection::system().await?;
        let proxy = ManagerProxy::new(&conn).await?;
        let mut stream = proxy.receive_prepare_for_sleep().await?;
        let (tx, rx) = mpsc::channel(1);
        tokio::spawn(async move {
            let _conn = conn;
            while let Some(signal) = stream.next().await {
                if let Ok(args) = signal.args() {
                    if !args.start() {
                        if tx.send(()).await.is_err() {
                            break;
                        }
                    }
                }
            }
        });
        Ok(rx)
    }

    pub async fn wait(&mut self) {
        match &mut self.rx {
            Some(rx) => { rx.recv().await; }
            None => std::future::pending::<()>().await,
        }
    }
}
