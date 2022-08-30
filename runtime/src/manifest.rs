#[macro_export]
macro_rules! manifest {
    ( $manifest:ident: [ $( $service:ident ),* ] ) => {
        struct $manifest {
            $( $service: < $service as ::micro_tower::core::service::CreateService > :: Service ),*
        }

        impl $manifest {
            pub fn create() -> Self {
                Self {
                    $( $service: < $service as ::micro_tower::core::service::CreateService > :: create() ),*
                }
            }
        }
    };
}
