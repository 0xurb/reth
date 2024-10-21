//! Type-safe abstractions for Dynamically Loaded ExExes

use std::{
    env,
    mem,
    path::{Path, PathBuf},
};

use eyre::Result;
use reth_node_api::FullNodeComponents;

use crate::{launch::BoxedLaunchExEx, ExExContext};

/// Required name of a user-defined function.
const USER_FN_NAME: &[u8] = b"_launch_exex";
/// This platform dynamic libraries prefix
const DYLIB_PREFIX: &str = env::consts::DLL_PREFIX;
/// This platform dynamic libraries suffix
const DYLIB_EXTENSION: &str = env::consts::DLL_SUFFIX;

/// Dynamically loads an ExEx entrypoint, which accepts a user-defined function representing the
/// core ExEx logic. The provided function must take an [`ExExContext`](`crate::ExExContext`) as its
/// argument.
///
/// # Returns
/// A Future that will be polled by the [`ExExManager`](`crate::ExExManager`).
///
/// ## Example usage:
/// ```rust
/// use reth_exex::{define_exex, ExExContext};
/// use reth_node_api::FullNodeComponents;
/// use std::future::Future;
///
/// // Create a function to produce ExEx logic
/// async fn exex<Node: FullNodeComponents>(
///     _ctx: ExExContext<Node>,
/// ) -> eyre::Result<impl std::future::Future<Output = eyre::Result<()>>> {
///     let _exex = async move { Ok(()) };
///     Ok(_exex)
/// }
///
/// // Use the macro to generate the entrypoint function
/// define_exex!(exex);
/// ```
#[macro_export]
macro_rules! define_exex {
    ($user_fn:ident) => {
        #[no_mangle]
        pub extern fn _launch_exex<Node: FullNodeComponents>(
            ctx: $crate::ExExContext<Node>,
        ) -> impl std::future::Future<
            Output = eyre::Result<impl Future<Output = eyre::Result<()>> + Send>,
        > {
            $user_fn(ctx)
        }
    };
}

/// Walks through a given directory path and loads all dynamic
/// libraries from that directory.
///
/// TODO(0xurb) - avoid duplicates
/// TODO(0xurb) - error on nested folders in libs/*
pub fn load_library_paths(
    directory: impl AsRef<Path>,
) -> Result<Vec<(String, PathBuf)>> {
    let directory = directory.as_ref();
    if !directory.is_dir() {
        eyre::bail!("Provided path is not a directory");
    }

    let paths = reth_fs_util::read_dir(directory)?.flat_map(|dir_entry| {
        let entry = dir_entry.unwrap(); // TODO(0xurb) - don't unwrap here
        let path = entry.path();

        // Check if the entry is a file and has a valid extension
        if path.is_file() {
            if let (Some(name), Some(ext)) = (path.file_name(), path.extension()) {
                if ext == DYLIB_EXTENSION {
                    // ExEx id is the name of dylib file
                    if let Some(Some(exex_id)) = name.to_str().map(|s| s.strip_prefix(DYLIB_PREFIX))
                    {
                        return Some((exex_id.to_owned(), path));
                    }
                }
            }
        }

        None
    });

    Ok(paths.collect())
}

/// TODO(0xurb) - safety section
pub unsafe fn load_library<Node: FullNodeComponents>(
    path: impl AsRef<Path>,
    ctx: ExExContext<Node>,
) -> Result<Box<dyn BoxedLaunchExEx<Node>>> {
    let lib = libloading::Library::new(path.as_ref())?;
    let raw_func_pointer: libloading::Symbol<
        '_,
        unsafe fn(ExExContext<Node>) -> *mut dyn BoxedLaunchExEx<Node>,
    > = lib.get(USER_FN_NAME)?;

    let exex: Box<_> = Box::from_raw(raw_func_pointer(ctx));

    mem::forget(lib);

    Ok(exex)
}
