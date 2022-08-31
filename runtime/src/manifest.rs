#[macro_export]
macro_rules! manifest {
    ( $manifest:ident: [ $( $service:ident ),* ] ) => {
        struct $manifest {
            $( $service: < $service as ::micro_tower::core::service::Create > :: Service ),*
        }

        impl $crate::manifest::Create for $manifest {
            fn create() -> Self {
                Self {
                    $( $service: < $service as ::micro_tower::core::service::Create > :: create() ),*
                }
            }
        }
    };
}

pub trait Create {
    fn create() -> Self;
}
