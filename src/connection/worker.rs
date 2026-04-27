use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use crate::connection::ConnectionState;
use crate::connection::establish::EstablishParams;
use crate::connection::execute;
use crate::{OracleArguments, OracleQueryResult, OracleRow, OracleStatement};
use crossfire::{AsyncTx, spsc};
use either::Either;
use futures_channel::oneshot;
use rbdc::Error;
use std::sync::Mutex;

pub(crate) struct ConnectionWorker {
    command_tx: AsyncTx<crossfire::spsc::Array<Command>>,
    pub(crate) shared: Arc<WorkerSharedState>,
}

pub(crate) struct WorkerSharedState {
    pub(crate) cached_statements_size: AtomicUsize,
    pub(crate) conn: Mutex<ConnectionState>,
}

pub enum Command {
    Prepare {
        query: Box<str>,
        tx: oneshot::Sender<Result<OracleStatement, Error>>,
    },
    Execute {
        query: Box<str>,
        arguments: Option<OracleArguments>,
        persistent: bool,
        tx: crossfire::Tx<
            crossfire::spsc::Array<Result<Either<OracleQueryResult, OracleRow>, Error>>,
        >,
    },
    ClearCache {
        tx: oneshot::Sender<()>,
    },
    Ping {
        tx: oneshot::Sender<Result<(), Error>>,
    },
    Shutdown {
        tx: oneshot::Sender<()>,
    },
}

impl ConnectionWorker {
    pub(crate) async fn establish(params: EstablishParams) -> Result<Self, Error> {
        let (establish_tx, establish_rx) = oneshot::channel();

        thread::Builder::new()
            .name(params.thread_name.clone())
            .spawn(move || {
                let (command_tx, command_rx) =
                    spsc::bounded_async_blocking(params.command_channel_size);

                let conn = match params.establish() {
                    Ok(conn) => conn,
                    Err(e) => {
                        establish_tx.send(Err(e)).ok();
                        return;
                    }
                };

                let shared = Arc::new(WorkerSharedState {
                    cached_statements_size: AtomicUsize::new(0),
                    conn: Mutex::new(conn),
                });
                let mut conn = shared.conn.lock().unwrap();

                if establish_tx
                    .send(Ok(Self {
                        command_tx,
                        shared: Arc::clone(&shared),
                    }))
                    .is_err()
                {
                    return;
                }

                loop {
                    let cmd = match command_rx.recv() {
                        Ok(cmd) => cmd,
                        Err(_) => break,
                    };

                    match cmd {
                        Command::Prepare { query, tx } => {
                            tx.send(prepare(&mut conn, &query).map(|prepared| {
                                update_cached_statements_size(
                                    &conn,
                                    &shared.cached_statements_size,
                                );
                                prepared
                            }))
                            .ok();
                        }
                        Command::Execute {
                            query,
                            arguments,
                            persistent,
                            tx,
                        } => {
                            let iter = match execute::iter(&mut conn, &query, arguments, persistent)
                            {
                                Ok(iter) => iter,
                                Err(e) => {
                                    tx.send(Err(e)).ok();
                                    continue;
                                }
                            };

                            for res in iter {
                                if tx.send(res).is_err() {
                                    break;
                                }
                            }

                            update_cached_statements_size(&conn, &shared.cached_statements_size);
                        }
                        Command::ClearCache { tx } => {
                            conn.statements.clear();
                            update_cached_statements_size(&conn, &shared.cached_statements_size);
                            tx.send(()).ok();
                        }
                        Command::Ping { tx } => {
                            let result = conn
                                .handle
                                .connection()
                                .ping()
                                .map_err(|e| Error::from(e.to_string()));
                            let should_stop = result.is_err();
                            tx.send(result).ok();
                            if should_stop {
                                return;
                            }
                        }
                        Command::Shutdown { tx } => {
                            let _ = conn.handle.connection().commit();
                            let _ = conn.handle.connection().close();
                            drop(conn);
                            drop(shared);
                            let _ = tx.send(());
                            return;
                        }
                    }
                }
            })
            .map_err(|e| Error::from(e.to_string()))?;

        establish_rx
            .await
            .map_err(|_| Error::from("WorkerCrashed"))?
    }

    pub(crate) async fn prepare(&mut self, query: &str) -> Result<OracleStatement, Error> {
        self.oneshot_cmd(|tx| Command::Prepare {
            query: query.into(),
            tx,
        })
        .await?
    }

    pub(crate) async fn execute(
        &mut self,
        query: String,
        args: Option<OracleArguments>,
        chan_size: usize,
        persistent: bool,
    ) -> Result<
        crossfire::AsyncRx<
            crossfire::spsc::Array<Result<Either<OracleQueryResult, OracleRow>, Error>>,
        >,
        Error,
    > {
        let (tx, rx) = spsc::bounded_blocking_async(chan_size);

        self.command_tx
            .send(Command::Execute {
                query: query.into(),
                arguments: args.map(OracleArguments::into_static),
                persistent,
                tx,
            })
            .await
            .map_err(|_| Error::from("WorkerCrashed"))?;

        Ok(rx)
    }

    pub(crate) async fn ping(&mut self) -> Result<(), Error> {
        self.oneshot_cmd(|tx| Command::Ping { tx }).await?
    }

    pub(crate) async fn oneshot_cmd<F, T>(&mut self, command: F) -> Result<T, Error>
    where
        F: FnOnce(oneshot::Sender<T>) -> Command,
    {
        let (tx, rx) = oneshot::channel();

        self.command_tx
            .send(command(tx))
            .await
            .map_err(|_| Error::from("WorkerCrashed"))?;

        rx.await.map_err(|_| Error::from("WorkerCrashed"))
    }

    pub(crate) async fn clear_cache(&mut self) -> Result<(), Error> {
        self.oneshot_cmd(|tx| Command::ClearCache { tx }).await
    }

    pub(crate) async fn shutdown(&mut self) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();

        self.command_tx
            .send(Command::Shutdown { tx })
            .await
            .map_err(|_| Error::from("WorkerCrashed"))?;

        rx.await.map_err(|_| Error::from("WorkerCrashed"))
    }
}

fn update_cached_statements_size(conn: &ConnectionState, size: &AtomicUsize) {
    size.store(conn.statements.len(), Ordering::Release);
}

fn prepare(conn: &mut ConnectionState, query: &str) -> Result<OracleStatement, Error> {
    super::executor::prepare(conn, query)
}
