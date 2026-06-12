#[macro_export]
macro_rules! require_auth {
    ($address:expr) => {
        $address.require_auth();
    };
}

#[macro_export]
macro_rules! require_auth_for_args {
    ($address:expr, $args:expr) => {
        $address.require_auth_for_args($args);
    };
}
