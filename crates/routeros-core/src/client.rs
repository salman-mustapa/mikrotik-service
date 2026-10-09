use std::collections::HashMap;
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex as StdMutex, Weak};
use std::task::{Context, Poll};

use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, Stream, StreamExt};
use md5::{Digest, Md5};
use tokio::net::{TcpStream, ToSocketAddrs};
use tokio::sync::{mpsc, Mutex};
use tokio_util::codec::Framed;

use crate::codec::{Codec, Kind, Sentence};
use crate::error::{Error, Result};

type Tx = mpsc::UnboundedSender<Result<Sentence>>;
type Rx = mpsc::UnboundedReceiver<Result<Sentence>>;
type Wire = Framed<TcpStream, Codec>;

struct Inner {
    sink: Mutex<SplitSink<Wire, Vec<String>>>,
    /// In-flight commands, keyed by `.tag`. This is what lets many commands
    /// (including long-running `listen`s) share one TCP connection.
    pending: StdMutex<HashMap<u32, Tx>>,
    next_tag: AtomicU32,
    closed: AtomicBool,
}

impl Inner {
    async fn send_tagged(&self, mut words: Vec<String>, tx: Tx) -> Result<u32> {
        if self.closed.load(Ordering::Acquire) {
            return Err(Error::Closed);
        }
        let tag = self.next_tag.fetch_add(1, Ordering::Relaxed);
        words.push(format!(".tag={tag}"));
        self.pending.lock().unwrap().insert(tag, tx);
        if let Err(e) = self.sink.lock().await.send(words).await {
            self.pending.lock().unwrap().remove(&tag);
            return Err(e);
        }
        Ok(tag)
    }

    /// Mark the connection dead and fail every in-flight command.
    fn fail(&self, reason: &str) {
        self.closed.store(true, Ordering::Release);
        let drained: Vec<Tx> = self.pending.lock().unwrap().drain().map(|(_, v)| v).collect();
        for tx in drained {
            let _ = tx.send(Err(Error::Fatal(reason.to_owned())));
        }
    }
}

async fn read_loop(mut stream: SplitStream<Wire>, inner: Weak<Inner>) {
    // Holding only a Weak means dropping the last `Client` lets this task exit
    // (on the next incoming sentence or when the router closes the socket).
    while let Some(item) = stream.next().await {
        let Some(inner) = inner.upgrade() else { return };
        match item {
            Ok(s) if s.kind == Kind::Fatal => {
                inner.fail(s.get("message").unwrap_or("fatal"));
                return;
            }
            Ok(s) => {
                let Some(tag) = s.tag else { continue };
                let mut pending = inner.pending.lock().unwrap();
                if s.kind == Kind::Done {
                    if let Some(tx) = pending.remove(&tag) {
                        let _ = tx.send(Ok(s));
                    }
                } else if let Some(tx) = pending.get(&tag) {
                    if tx.send(Ok(s)).is_err() {
                        pending.remove(&tag); // receiver gone
                    }
                }
            }
            Err(e) => {
                inner.fail(&e.to_string());
                return;
            }
        }
    }
    if let Some(inner) = inner.upgrade() {
        inner.fail("connection closed by peer");
    }
}

/// Turn `("/ip/address/print", [("?disabled","no"), (".proplist","address")])`
/// into RouterOS API words.
///
/// * keys starting with `?` become query words (`?disabled=no`)
/// * keys starting with `.` are sent as-is (`.proplist=address`)
/// * everything else becomes an attribute word (`=name=value`)
pub fn build_command<K, V>(command: &str, args: impl IntoIterator<Item = (K, V)>) -> Vec<String>
where
    K: AsRef<str>,
    V: AsRef<str>,
{
    let mut words = vec![command.to_owned()];
    for (k, v) in args {
        let (k, v) = (k.as_ref(), v.as_ref());
        if k.starts_with('?') || k.starts_with('.') {
            if v.is_empty() {
                words.push(k.to_owned());
            } else {
                words.push(format!("{k}={v}"));
            }
        } else {
            words.push(format!("={k}={v}"));
        }
    }
    words
}

fn trap_error(s: &Sentence) -> Error {
    Error::Trap {
        message: s.get("message").unwrap_or_default().to_owned(),
        category: s.get("category").map(str::to_owned),
    }
}

/// A multiplexed connection to one router. Cheap to clone.
#[derive(Clone)]
pub struct Client {
    inner: Arc<Inner>,
}

impl Client {
    /// Connect and log in. Supports RouterOS >= 6.43 (plain) and older (MD5 challenge).
    pub async fn connect(addr: impl ToSocketAddrs, user: &str, password: &str) -> Result<Self> {
        let tcp = TcpStream::connect(addr).await?;
        tcp.set_nodelay(true)?;
        let (sink, stream) = Framed::new(tcp, Codec).split();
        let inner = Arc::new(Inner {
            sink: Mutex::new(sink),
            pending: StdMutex::new(HashMap::new()),
            next_tag: AtomicU32::new(1),
            closed: AtomicBool::new(false),
        });
        tokio::spawn(read_loop(stream, Arc::downgrade(&inner)));

        let client = Client { inner };
        client.login(user, password).await?;
        Ok(client)
    }

    async fn login(&self, user: &str, password: &str) -> Result<()> {
        let map_login = |e: Error| match e {
            Error::Trap { message, .. } => Error::Login(message),
            other => other,
        };

        let (_, done) = self
            .run_full(build_command("/login", [("name", user), ("password", password)]))
            .await
            .map_err(map_login)?;

        // Pre-6.43 routers ignore the credentials above and answer with a challenge.
        if let Some(ret) = done.get("ret") {
            let challenge = hex::decode(ret).map_err(|e| Error::Login(e.to_string()))?;
            let mut md5 = Md5::new();
            md5.update([0u8]);
            md5.update(password.as_bytes());
            md5.update(&challenge);
            let response = format!("00{}", hex::encode(md5.finalize()));
            self.run_full(build_command("/login", [("name", user), ("response", response.as_str())]))
                .await
                .map_err(map_login)?;
        }
        Ok(())
    }

    pub fn is_closed(&self) -> bool {
        self.inner.closed.load(Ordering::Acquire)
    }

    /// Run a command and collect all `!re` rows. A `!trap` becomes [`Error::Trap`].
    pub async fn run(&self, words: Vec<String>) -> Result<Vec<Sentence>> {
        self.run_full(words).await.map(|(rows, _)| rows)
    }

    async fn run_full(&self, words: Vec<String>) -> Result<(Vec<Sentence>, Sentence)> {
        let (tx, mut rx) = mpsc::unbounded_channel();
        self.inner.send_tagged(words, tx).await?;

        let mut rows = Vec::new();
        let mut trap = None;
        loop {
            let recv_res = tokio::time::timeout(std::time::Duration::from_secs(15), rx.recv()).await;
            match recv_res {
                Ok(Some(msg)) => {
                    let s = msg?;
                    match s.kind {
                        Kind::Re => rows.push(s),
                        Kind::Trap => trap = Some(s),
                        Kind::Done => {
                            return match trap {
                                Some(t) => Err(trap_error(&t)),
                                None => Ok((rows, s)),
                            };
                        }
                        Kind::Empty | Kind::Fatal => {}
                    }
                }
                Ok(None) => return Err(Error::Closed),
                Err(_) => return Err(Error::Fatal("Command execution timed out after 15 seconds".into())),
            }
        }
    }

    /// Start a long-running command (`/listen`, `monitor-traffic`, `follow`, ...).
    /// Dropping the returned [`Subscription`] sends `/cancel` to the router.
    pub async fn listen(&self, words: Vec<String>) -> Result<Subscription> {
        let (tx, rx) = mpsc::unbounded_channel();
        let tag = self.inner.send_tagged(words, tx).await?;
        Ok(Subscription {
            rx,
            tag,
            inner: self.inner.clone(),
            finished: false,
        })
    }
}

/// Stream of `!re` rows from a long-running command.
pub struct Subscription {
    rx: Rx,
    tag: u32,
    inner: Arc<Inner>,
    finished: bool,
}

impl Stream for Subscription {
    type Item = Result<Sentence>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match self.rx.poll_recv(cx) {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Ready(Some(Err(e))) => {
                    self.finished = true;
                    return Poll::Ready(Some(Err(e)));
                }
                Poll::Ready(Some(Ok(s))) => match s.kind {
                    Kind::Re => return Poll::Ready(Some(Ok(s))),
                    Kind::Trap => {
                        return Poll::Ready(Some(Err(trap_error(&s))));
                    }
                    Kind::Done => {
                        self.finished = true;
                        return Poll::Ready(None);
                    }
                    Kind::Empty | Kind::Fatal => continue,
                },
            }
        }
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        if self.finished || self.inner.closed.load(Ordering::Acquire) {
            return;
        }
        self.inner.pending.lock().unwrap().remove(&self.tag);
        let (inner, tag) = (self.inner.clone(), self.tag);
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn(async move {
                // The reply to /cancel (and the interrupted command's trailing
                // `!trap`/`!done`) are intentionally ignored by the reader.
                let (tx, _rx) = mpsc::unbounded_channel();
                let _ = inner
                    .send_tagged(vec!["/cancel".to_owned(), format!("=tag={tag}")], tx)
                    .await;
            });
        }
    }
}
