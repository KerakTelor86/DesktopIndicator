#[macro_export]
macro_rules! guard_clause {
    ($x:expr, $err:ident, $body:block) => {
        match $x {
            Ok(x) => x,
            Err($err) => $body,
        }
    };

    ($x:expr, $body:block) => {
        match $x {
            Ok(x) => x,
            Err(_) => $body,
        }
    };
}
