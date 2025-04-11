use custom_error::custom_error;

use crate::error::WhError;

custom_error! {pub RemoveInode
    WhError{source: WhError} = "{source}",
}
