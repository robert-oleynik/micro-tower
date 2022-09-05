#[macro_export]
macro_rules! manifest {
    ( $manifest:ident: [ $( $service:ident ),* $(,)? ] ) => {
        #[derive(::std::clone::Clone)]
        struct $manifest {
            $( $service: < $service as ::micro_tower::service::Create > :: Service ),*
        }

        impl $crate::manifest::Create for $manifest {
            fn create() -> Self {
                let span = $crate::tracing::span!($crate::tracing::Level::INFO, "init");
                let _guard = span.enter();

                let mut registry = $crate::utils::TypeRegistry::new();

                let mut empty = false;
                let mut changed = true;
                while !empty && changed {
                    $crate::tracing::event!($crate::tracing::Level::TRACE, "start cycle");
                    empty = true;
                    changed = false;
                    $(
                        if !registry.contains_key(&<$service as $crate::utils::Named>::name()) {
                            empty = false;
                            let service = ::std::stringify!($service);
                            if <$service as $crate::service::Create>::deps().iter().all(|dep| registry.contains_key(dep)) {
                                changed = true;
                                registry.insert(
                                    <$service as $crate::utils::Named>::name(),
                                    Box::new(<$service as $crate::service::Create>::create(&registry))
                                    );
                                $crate::tracing::event!($crate::tracing::Level::INFO, service);
                            } else {
                                $crate::tracing::event!($crate::tracing::Level::DEBUG, message = "delay init due to missing dependencies of", service);
                            }
                        }
                    )*
                }
                if !empty {
                    panic!("Contains cyclic dependency");
                }

                Self {
                    $( $service: <$crate::utils::TypeRegistry as $crate::service::GetByName<$service>>::get(&registry).unwrap().into_inner()),*
                }
            }
        }
    };
}

pub trait Create {
    fn create() -> Self;
}
