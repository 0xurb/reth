//! Dynamically loading type-safe abstractions

/// Dynamically loaded ExEx entrypoint, that accepts the [`ExExContext`](`crate::ExExContext`)
/// and returns a Future that will be polled by the [`ExExManager`](`crate::ExExManager`).
///
/// ## Example usage:
/// ```rust
/// use std::future::Future;
/// use reth_exex::{define_exex, ExExContext};
/// use reth_node_api::FullNodeComponents;
///
/// // Create a function to produce ExEx logic
/// async fn exex<Node: FullNodeComponents>(_ctx: ExExContext<Node>) -> eyre::Result<()> {
///     Ok(())
/// }
///
/// // Use the macro to generate the entrypoint function
/// define_exex!(exex,<Node>);
/// ```
#[macro_export]
macro_rules! define_exex {
    ($user_fn:ident,<$node:ident>) => {
        #[no_mangle]
        pub extern "C" fn _launch_exex<$node: FullNodeComponents>(
            ctx: $crate::ExExContext<$node>,
        ) -> impl std::future::Future<
            Output = eyre::Result<impl Future<Output = eyre::Result<()>> + Send>,
        > {
            async move { Ok($user_fn(ctx)) }
        }
    };
}
