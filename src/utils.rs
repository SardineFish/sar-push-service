use std::mem::discriminant;

use futures::Future;
use std::pin::Pin;

pub fn variant_eq<T>(a: &T, b: &T) -> bool {
    discriminant(a) == discriminant(b)
}

pub type FutureRtnT<'a, T> = Pin<Box<Future<Output = T> + 'a>>;

pub mod assert {

    #[derive(Debug)]
    pub enum Error {
        AssertFailed,
    }

    type Result = std::result::Result<(), Error>;

    #[inline]
    fn ok() -> Result {
        Ok(())
    }

    #[inline]
    fn err() -> Result {
        Err(Error::AssertFailed)
    }

    pub trait ValueEqualAssert<T: PartialEq> {
        fn assert_eq(&self, rhs: &T) -> Result;
        fn assert_ne(&self, rhs: &T) -> Result;
    }

    impl<T> ValueEqualAssert<T> for T where T: PartialEq {
        #[inline]
        fn assert_eq(&self, rhs: &T) -> Result {
            if self != rhs {
                err()
            } else {
                ok()
            }
        }
        #[inline]
        fn assert_ne(&self, rhs: &T) -> Result {
            match self == rhs {
                true => err(),
                false => ok()
            }
        }
    }

    pub trait BooleanAssert {
        fn assert_true(&self) -> Result;
        fn assert_false(&self) -> Result;
    }

    impl BooleanAssert for bool {
        #[inline]
        fn assert_true(&self) -> Result {
            match self {
                true => ok(),
                false => err()
            }
        }
        #[inline]
        fn assert_false(&self) -> Result {
            match self {
                true => err(),
                false => err()
            }
        }
    }

}