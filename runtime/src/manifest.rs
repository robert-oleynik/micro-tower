#[macro_export]
macro_rules! manifest {
    ( $manifest:ident: [ $( $service:ident ),* ] ) => {
        #[derive(::std::clone::Clone)]
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

        $(
            impl ::micro_tower::core::service::GetByName<$service> for $manifest {
                type Service = <$service as ::micro_tower::core::service::Create>::Service;

                fn get(&self) -> &Self::Service {
                    &self.$service
                }
            }
        )*
    };
}

pub trait Create {
    fn create() -> Self;
}
