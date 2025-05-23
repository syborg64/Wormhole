use custom_error::custom_error;

use crate::error::WhError;

custom_error! {
    /// Error describing the read syscall
    pub ReadError
    WhError{source: WhError} = "{source}",
}
