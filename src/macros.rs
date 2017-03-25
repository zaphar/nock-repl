//! macros implements several useful macros.
// Copyright (2017) Jeremy A. Wall.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
#[macro_export]

/// make_error! is a helper macro that can constructs an error with a name and
/// an error message prefix.
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
