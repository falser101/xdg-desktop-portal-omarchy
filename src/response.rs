use zvariant::{SerializeDict, Type};

pub const SUCCESS: u32 = 0;
pub const CANCELLED: u32 = 1;
pub const OTHER: u32 = 2;

/// `(u, a{sv})` reply used by most backend methods.
#[derive(Debug, Type)]
#[zvariant(signature = "(ua{sv})")]
pub enum PortalResponse<T: Type + serde::Serialize> {
    Success(T),
    Cancelled,
    Other,
}

impl<T: Type + serde::Serialize> serde::Serialize for PortalResponse<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Success(res) => (SUCCESS, res).serialize(serializer),
            Self::Cancelled => (CANCELLED, EmptyDict::default()).serialize(serializer),
            Self::Other => (OTHER, EmptyDict::default()).serialize(serializer),
        }
    }
}

#[derive(Default, SerializeDict, Type)]
#[zvariant(signature = "dict")]
pub struct EmptyDict {}
