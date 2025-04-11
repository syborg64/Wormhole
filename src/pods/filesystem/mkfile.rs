use custom_error::custom_error;

use crate::error::WhError;

custom_error! {pub MakeInode
    WhError{source: WhError} = "{source}",
    AlreadyExist = "File already existing",
    ParentNotFound = "Parent does not exist",
    ParentNotFolder = "Parent isn't a folder",
    LocalCreationFailed{io: std::io::Error} = "Local creation failed: {io}"
}
