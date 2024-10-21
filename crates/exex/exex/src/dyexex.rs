//! Type-safe abstractions for Dynamically Loaded ExExes

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
        #[allow(no_mangle_generic_items, unreachable_pub)]
        #[no_mangle]
        pub extern "Rust" fn _launch_exex<Node: FullNodeComponents>(
            ctx: $crate::ExExContext<Node>,
        ) -> impl std::future::Future<
            Output = eyre::Result<impl std::future::Future<Output = eyre::Result<()>> + Send>,
        > {
            $user_fn(ctx)
        }
    };
    (async move |ctx| {
        Ok($user_fn:ident(ctx))
    }) => {
        #[allow(no_mangle_generic_items, unreachable_pub)]
        #[no_mangle]
        pub async extern "Rust" fn _launch_exex<Node: FullNodeComponents>(
            ctx: $crate::ExExContext<Node>,
        ) -> eyre::Result<impl std::future::Future<Output = eyre::Result<()>> + Send> {
            Ok($user_fn(ctx))
        }
    };
}

#[cfg(test)]
mod tests {
    use reth_node_api::FullNodeComponents;

    use crate::context::ExExContext;

    #[test]
    const fn should_define_exex() {
        async fn _exex<Node: FullNodeComponents>(
            _ctx: ExExContext<Node>,
        ) -> eyre::Result<impl std::future::Future<Output = eyre::Result<()>>> {
            let _exex = async move { Ok(()) };
            Ok(_exex)
        }

        define_exex!(_exex);

        fn _it_defines<Node: FullNodeComponents>(
            ctx: ExExContext<Node>,
        ) -> impl std::future::Future<
            Output = eyre::Result<impl std::future::Future<Output = eyre::Result<()>> + Send>,
        > {
            _launch_exex::<Node>(ctx)
        }
    }

    #[test]
    const fn should_define_exex_closure() {
        async fn _exex<Node: FullNodeComponents>(_ctx: ExExContext<Node>) -> eyre::Result<()> {
            Ok(())
        }

        // ensure that we have right `exex` closure by design
        async fn __exex<Node: FullNodeComponents>(
            ctx: ExExContext<Node>,
        ) -> eyre::Result<impl std::future::Future<Output = eyre::Result<()>>> {
            Ok(_exex(ctx))
        }

        define_exex!(async move |ctx| { Ok(_exex(ctx)) });

        fn _it_defines<Node: FullNodeComponents>(
            ctx: ExExContext<Node>,
        ) -> impl std::future::Future<
            Output = eyre::Result<impl std::future::Future<Output = eyre::Result<()>> + Send>,
        > {
            _launch_exex::<Node>(ctx)
        }
    }
}
