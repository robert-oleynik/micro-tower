#[macro_export]
macro_rules! manifest {
    ( $manifest:ident: [ $( $service:ident ),* ] ) => {
        #[derive(::std::clone::Clone)]
        struct $manifest {
            $( $service: < $service as ::micro_tower::service::Create > :: Service ),*
        }

        impl $crate::manifest::Create for $manifest {
            fn create() -> Self {
                let mut registry = $crate::utils::TypeRegistry::new();

                let mut empty = false;
                let mut changed = true;
                while !empty && changed {
                    empty = true;
                    changed = false;
                    $(
                        if !registry.contains_key(&<$service as $crate::utils::Named>::name()) {
                            empty = false;
                            if <$service as $crate::service::Create>::deps().iter().all(|dep| registry.contains_key(dep)) {
                                changed = true;
                                registry.insert(
                                    <$service as $crate::utils::Named>::name(),
                                    Box::new(<$service as $crate::service::Create>::create(&registry))
                                    );
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
