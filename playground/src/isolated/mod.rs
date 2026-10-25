//! Dynamic UI registration and generation lifecycle proof, before windows open.
use tuffite::isolated::{Error, Host, Registration, WorldDefinition};
use tuffite::{ApplicationError, ApplicationResult};

fn error(error: impl std::fmt::Display) -> ApplicationError {
    ApplicationError::new(error.to_string())
}
async fn completed<T>(
    receive: futures_channel::oneshot::Receiver<Result<T, Error>>,
) -> ApplicationResult<T> {
    receive.await.map_err(error)?.map_err(error)
}
fn definition(name: &str) -> WorldDefinition {
    WorldDefinition::new(name, include_str!("card.js")).allow_origin("playground://main")
}

pub async fn register(host: &Host) -> ApplicationResult<()> {
    for name in ["weather", "weatherOther", "removed"] {
        let (send, receive) = futures_channel::oneshot::channel();
        host.register(
            WorldDefinition::new(name, "throw new Error('obsolete snapshot')")
                .allow_origin("playground://main"),
            move |result| {
                let _ = send.send(result);
            },
        )
        .map_err(error)?;
        let first: Registration = completed(receive).await?;
        let (send, receive) = futures_channel::oneshot::channel();
        host.replace(&first, definition(name), move |result| {
            let _ = send.send(result);
        })
        .map_err(error)?;
        let current = completed(receive).await?;
        let (send, receive) = futures_channel::oneshot::channel();
        host.unregister(&first, move |result| {
            let _ = send.send(result);
        })
        .map_err(error)?;
        if receive.await.map_err(error)?.is_ok() {
            return Err(error("stale isolated registration accepted"));
        }
        if current.generation() <= first.generation() {
            return Err(error("isolated generation did not advance"));
        }
        if name == "removed" {
            let (send, receive) = futures_channel::oneshot::channel();
            host.unregister(&current, move |result| {
                let _ = send.send(result);
            })
            .map_err(error)?;
            completed(receive).await?;
        }
    }
    Ok(())
}
