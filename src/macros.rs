//! macros implements several useful macros.

#[macro_export]
macro_rules! make_error {
    ($e:ident, $msg:expr) => {

        #[derive(Debug)]
        pub struct $e {
            msg: String,
            cause: Option<Box<error::Error>>,
        }


        impl $e {
            pub fn new<S: Into<String>>(msg: S) -> Self {
                $e {
                    msg: msg.into(),
                    cause: None,
                }
            }

            pub fn new_with_cause<S: Into<String>>(msg: S, err: Box<error::Error>) -> Self {
                $e {
                    msg: msg.into(),
                    cause: Some(err),
                }
            }
        }

        impl Display for $e {
            fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
                try!(write!(f, $msg, self.msg));
                if let Some(ref cause) = self.cause {
                    try!(write!(f, "Cause:\n\t{}", cause));
                }
                return Ok(());
            }
        }

        impl error::Error for $e {
            fn description(&self) -> &str {
                &self.msg
            }

            fn cause(&self) -> Option<&error::Error> {
                if let Some(ref cause) = self.cause {
                    return Some(cause.as_ref());
                }
                return None;
            }
        }
    };
}
