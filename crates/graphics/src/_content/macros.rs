#[macro_export]
macro_rules! impl_resource {
    ($type:ty, $params:ty, $loader:ty) => {
        // Реализация Resource
        impl $crate::content::resource::Resource for $type {
            fn type_id(&self) -> std::any::TypeId {
                std::any::TypeId::of::<Self>()
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }

            fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
                self
            }
        }

        // Реализация TypedResource
        impl $crate::content::resource::TypedResource<$params> for $type {
            type ResourceLoader = $loader;
        }
    };
}
