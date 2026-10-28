//! Dynamic registration and generation lifecycle proof, before windows open.
use tuffite::isolated::{Error as IsolatedError, Host, Registration, WorldDefinition};
use tuffite::{Error, Result};

async fn completed<T>(
    receive: futures_channel::oneshot::Receiver<Result<T, IsolatedError>>,
) -> Result<T> {
    Ok(receive.await.map_err(Error::from_source)??)
}
fn definition(name: &str) -> WorldDefinition {
    WorldDefinition::package(name, "probe.js")
        .resource("probe.js", b"export const version = 2;")
        .allow_origin("playground://main")
}

pub(super) async fn register(host: &Host) -> Result<()> {
    for name in ["dynamicProbe", "removed"] {
        let (send, receive) = futures_channel::oneshot::channel();
        host.register(
            WorldDefinition::new(name, "throw new Error('obsolete snapshot')")
                .allow_origin("playground://main"),
            move |result| {
                let _ = send.send(result);
            },
        )?;
        let first: Registration = completed(receive).await?;
        let (send, receive) = futures_channel::oneshot::channel();
        host.replace(&first, definition(name), move |result| {
            let _ = send.send(result);
        })?;
        let current = completed(receive).await?;
        let (send, receive) = futures_channel::oneshot::channel();
        host.unregister(&first, move |result| {
            let _ = send.send(result);
        })?;
        if receive.await.map_err(Error::from_source)?.is_ok() {
            return Err(Error::new("stale isolated registration accepted"));
        }
        if current.generation() <= first.generation() {
            return Err(Error::new("isolated generation did not advance"));
        }
        if name == "removed" {
            let (send, receive) = futures_channel::oneshot::channel();
            host.unregister(&current, move |result| {
                let _ = send.send(result);
            })?;
            completed(receive).await?;
        }
    }
    let (send, receive) = futures_channel::oneshot::channel();
    host.register(
        WorldDefinition::new("broken", "throw new Error('ESM fixture failure');")
            .allow_origin("playground://main"),
        move |result| {
            let _ = send.send(result);
        },
    )?;
    completed(receive).await?;
    Ok(())
}
