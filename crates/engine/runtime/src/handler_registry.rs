use std::sync::{Arc, Mutex};

#[derive(Default)]
pub struct HandlerRegistry<H: ?Sized + Send + Sync + 'static> {
    handlers: Mutex<Vec<Arc<H>>>,
}

impl<H: ?Sized + Send + Sync + 'static> HandlerRegistry<H> {
    pub fn new() -> Self {
        Self {
            handlers: Mutex::new(Vec::new()),
        }
    }

    pub fn register_arc(&self, handler: Arc<H>) {
        self.handlers
            .lock()
            .expect("handler registry mutex should not be poisoned")
            .push(handler);
    }

    pub fn handlers(&self) -> Vec<Arc<H>> {
        self.handlers
            .lock()
            .expect("handler registry mutex should not be poisoned")
            .clone()
    }

    pub fn len(&self) -> usize {
        self.handlers().len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub trait RoutedHandler<Ctx, Cmd, Out>: Send + Sync {
    fn name(&self) -> &'static str;
    fn can_handle(&self, command: &Cmd) -> bool;
    fn handle(&self, ctx: &Ctx, command: Cmd) -> Out;
}

pub type RoutedHandlerRegistry<Ctx, Cmd, Out> = HandlerRegistry<dyn RoutedHandler<Ctx, Cmd, Out>>;

pub fn register_routed_handler<Ctx, Cmd, Out, H>(
    registry: &RoutedHandlerRegistry<Ctx, Cmd, Out>,
    handler: H,
) where
    H: RoutedHandler<Ctx, Cmd, Out> + 'static,
{
    registry.register_arc(Arc::new(handler));
}

pub struct HandlerDispatcher<H: ?Sized + Send + Sync + 'static> {
    registry: Arc<HandlerRegistry<H>>,
}

impl<H: ?Sized + Send + Sync + 'static> HandlerDispatcher<H> {
    pub fn new(registry: Arc<HandlerRegistry<H>>) -> Self {
        Self { registry }
    }

    pub fn dispatch_first<R>(&self, mut dispatch: impl FnMut(&H) -> Option<R>) -> Option<R> {
        for handler in self.registry.handlers() {
            if let Some(result) = dispatch(handler.as_ref()) {
                return Some(result);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{
        HandlerDispatcher, HandlerRegistry, RoutedHandler, RoutedHandlerRegistry,
        register_routed_handler,
    };

    trait TestHandler: Send + Sync {
        fn name(&self) -> &'static str;
    }

    struct ExampleHandler;

    impl TestHandler for ExampleHandler {
        fn name(&self) -> &'static str {
            "example"
        }
    }

    impl RoutedHandler<(), &'static str, &'static str> for ExampleHandler {
        fn name(&self) -> &'static str {
            "example"
        }

        fn can_handle(&self, command: &&'static str) -> bool {
            *command == "example"
        }

        fn handle(&self, _ctx: &(), command: &'static str) -> &'static str {
            command
        }
    }

    #[test]
    fn stores_trait_object_handlers() {
        let mut registry = HandlerRegistry::<dyn TestHandler>::new();
        registry.register_arc(Arc::new(ExampleHandler));

        assert_eq!(registry.handlers().len(), 1);
        assert_eq!(registry.handlers()[0].name(), "example");
    }

    #[test]
    fn dispatches_first_matching_handler() {
        let mut registry = HandlerRegistry::<dyn TestHandler>::new();
        registry.register_arc(Arc::new(ExampleHandler));

        let dispatched = HandlerDispatcher::new(Arc::new(registry))
            .dispatch_first(|handler| (handler.name() == "example").then_some(handler.name()));

        assert_eq!(dispatched, Some("example"));
    }

    #[test]
    fn stores_and_dispatches_routed_handlers() {
        let mut registry = RoutedHandlerRegistry::<(), &'static str, &'static str>::new();
        register_routed_handler(&mut registry, ExampleHandler);

        let dispatched = HandlerDispatcher::new(Arc::new(registry)).dispatch_first(|handler| {
            handler
                .can_handle(&"example")
                .then(|| handler.handle(&(), "example"))
        });

        assert_eq!(dispatched, Some("example"));
    }
}
