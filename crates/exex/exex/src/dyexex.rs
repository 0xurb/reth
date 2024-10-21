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
        #[no_mangle]
        pub extern fn _launch_exex<Node: FullNodeComponents>(
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
        #[no_mangle]
        pub extern fn _launch_exex<Node: FullNodeComponents>(
            ctx: $crate::ExExContext<Node>,
        ) -> impl std::future::Future<
            Output = eyre::Result<impl std::future::Future<Output = eyre::Result<()>> + Send>,
        > {
            async move { Ok($user_fn(ctx)) }
        }
    };
}

#[cfg(test)]
mod tests {
    use reth_exex_test_utils::{test_exex_context, Adapter};
    use reth_node_api::FullNodeComponents;

    use crate::ExExContext;

    #[tokio::test]
    async fn should_define_exex() -> eyre::Result<()> {
        async fn exex<Node: FullNodeComponents>(
            _ctx: ExExContext<Node>,
        ) -> eyre::Result<impl std::future::Future<Output = eyre::Result<()>>> {
            let _exex = async move { Ok(()) };
            Ok(_exex)
        }

        define_exex!(exex);

        let (ctx, _) = test_exex_context().await?;

        _launch_exex::<Adapter>(ctx);

        Ok(())
    }

    #[tokio::test]
    async fn should_define_exex_closure() -> eyre::Result<()> {
        // ensure that we have right `exex` closure by design
        async fn _exex<Node: FullNodeComponents>(
            ctx: ExExContext<Node>,
        ) -> eyre::Result<impl std::future::Future<Output = eyre::Result<()>>> {
            Ok(exex(ctx))
        }

        async fn exex<Node: FullNodeComponents>(_ctx: ExExContext<Node>) -> eyre::Result<()> {
            Ok(())
        }

        define_exex!(async move |ctx| { Ok(exex(ctx)) });

        let (ctx, _) = test_exex_context().await?;

        _launch_exex::<Adapter>(ctx);

        Ok(())
    }
}
