pub mod message;
pub mod task;
pub mod transport;
pub mod user;

pub use message::*;
pub use task::*;
pub use transport::*;
pub use user::*;

pub fn is_not_found_record_err(err: &anyhow::Error) -> bool {
    return match err.downcast_ref::<sqlx::Error>() {
        None => false,
        Some(err) => match err {
            sqlx::Error::RowNotFound => true,
            _ => false,
        },
    };
}
