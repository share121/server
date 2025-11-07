#[macro_export]
macro_rules! send_err {
    ($expr:expr, $tx:expr, $event:expr) => {
        match $expr {
            Ok(r) => r,
            Err(e) => {
                $tx.send($event(Err(e))).await.unwrap();
                return;
            }
        }
    };
}
#[macro_export]
macro_rules! send_err2 {
    ($expr:expr, $tx:expr, $event:expr) => {
        match $expr {
            Ok(r) => r,
            Err(e) => {
                $tx.send($event(e)).await.unwrap();
                return;
            }
        }
    };
}
