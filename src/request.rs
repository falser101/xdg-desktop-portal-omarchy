use crate::response::PortalResponse;
use tokio_util::sync::CancellationToken;
use zbus::object_server::Interface;
use zbus::zvariant::{ObjectPath, Type};
use zbus::Connection;

pub struct Request {
    token: CancellationToken,
}

#[zbus::interface(name = "org.freedesktop.impl.portal.Request")]
impl Request {
    async fn close(&self) {
        self.token.cancel();
    }
}

pub struct Session;

#[zbus::interface(name = "org.freedesktop.impl.portal.Session")]
impl Session {
    async fn close(&self, #[zbus(signal_emitter)] emitter: zbus::object_server::SignalEmitter<'_>) {
        let _ = Self::closed(&emitter).await;
        let _ = emitter
            .connection()
            .object_server()
            .remove::<Self, _>(emitter.path())
            .await;
    }

    #[zbus(signal)]
    async fn closed(emitter: &zbus::object_server::SignalEmitter<'_>) -> zbus::Result<()>;

    #[zbus(property, name = "version")]
    fn version(&self) -> u32 {
        1
    }
}

/// Export a Request at `handle`. `Close()` cancels the token passed to `f`.
pub async fn with_request<T, F, Fut>(
    connection: &Connection,
    handle: &ObjectPath<'_>,
    f: F,
) -> PortalResponse<T>
where
    T: Type + serde::Serialize,
    F: FnOnce(CancellationToken) -> Fut,
    Fut: std::future::Future<Output = PortalResponse<T>>,
{
    let token = CancellationToken::new();
    let _ = connection
        .object_server()
        .at(handle, Request { token: token.clone() })
        .await;

    let result = f(token).await;

    let _ = connection
        .object_server()
        .remove::<Request, _>(handle)
        .await;
    result
}

pub async fn export_session(connection: &Connection, handle: &ObjectPath<'_>) -> bool {
    connection.object_server().at(handle, Session).await.unwrap_or(false)
}

pub async fn remove_interface<I: Interface>(connection: &Connection, handle: &ObjectPath<'_>) {
    let _ = connection.object_server().remove::<I, _>(handle).await;
}

pub async fn export_request(connection: &Connection, handle: &ObjectPath<'_>) -> CancellationToken {
    let token = CancellationToken::new();
    let _ = connection
        .object_server()
        .at(handle, Request { token: token.clone() })
        .await;
    token
}
