use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
};

use crate::{FnCallArg, Locked, RuntimeResult, Value};

pub trait ExternalFn: 'static {
    fn call(&mut self, name: &str, args: FnCallArg) -> RuntimeResult<Value>;
}

impl<T: FnMut(FnCallArg) -> RuntimeResult<Value> + 'static> ExternalFn for T {
    fn call(&mut self, _: &str, args: FnCallArg) -> RuntimeResult<Value> {
        self(args)
    }
}

#[derive(Debug, Clone)]
pub struct ExtractFn<T, F> {
    func: F,
    _marker: PhantomData<T>,
}

macro_rules! impl_fn {
    ($len:literal, $($ty:ident $(,)?)*) => {
        impl<Func, $( $ty ,)*> From<Func> for $crate::ExtractFn<($( $ty ,)*), Func>
        where
            $( $ty: $crate::FromValue + 'static ,)*
            Func: Fn($( $ty ,)*) -> $crate::RuntimeResult<$crate::Value> + 'static,
        {
            fn from(func: Func) -> Self {
                Self {
                    func,
                    _marker: ::std::marker::PhantomData,
                }
            }
        }

        impl<Func, $( $ty ,)*> $crate::ExternalFn for$crate:: ExtractFn<($( $ty ,)*), Func>
        where
            $( $ty:$crate:: FromValue + 'static ,)*
            Func: Fn($( $ty ,)*) -> $crate::RuntimeResult<$crate::Value> + 'static,
        {
            fn call(&mut self, name: &str, args: $crate::FnCallArg) ->$crate:: RuntimeResult<$crate::Value> {
                let len = $len;
                if args.len() != len {
                    return Err($crate::RuntimeError::ArgumentError {
                        ident: name.to_owned(),
                        expected: len,
                        found: args.len(),
                    });
                };
                #[allow(unused_mut)]
                #[allow(unused_variables)]
                let mut iter = args.into_iter().enumerate();
                (self.func)($({
                    let (i, arg) = iter.next().unwrap();
                    $ty::from_value(&format!("ExternalFn({name}) Arg#{i}"), arg)?
                },)*)
            }
        }
    };
}

#[rustfmt::skip]
mod impl_fns_without_fmt {
    impl_fn!(0,);
    impl_fn!(1, A,);
    impl_fn!(2, A, B);
    impl_fn!(3, A, B, C);
    impl_fn!(4, A, B, C, D);
    impl_fn!(5, A, B, C, D, E);
    impl_fn!(6, A, B, C, D, E, F);
    impl_fn!(7, A, B, C, D, E, F, G);
    impl_fn!(8, A, B, C, D, E, F, G, H);
    impl_fn!(9, A, B, C, D, E, F, G, H, I);
    impl_fn!(10, A, B, C, D, E, F, G, H, I, J);
    impl_fn!(11, A, B, C, D, E, F, G, H, I, J, K);
    impl_fn!(12, A, B, C, D, E, F, G, H, I, J, K, L);
    impl_fn!(13, A, B, C, D, E, F, G, H, I, J, K, L, M);
    impl_fn!(14, A, B, C, D, E, F, G, H, I, J, K, L, M, N);
    impl_fn!(15, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O);
    impl_fn!(16, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P);
    impl_fn!(17, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q);
    impl_fn!(18, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R);
    impl_fn!(19, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S);
    impl_fn!(20, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T);
    impl_fn!(21, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U);
    impl_fn!(22, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V);
    impl_fn!(23, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W);
    impl_fn!(24, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X);
}

pub struct NativeFn {
    ptr: Locked<Box<dyn ExternalFn>>,
    name: String,
}

impl NativeFn {
    pub fn new(ptr: impl ExternalFn, name: impl Into<String>) -> Self {
        Self {
            ptr: Locked::new(Box::new(ptr)),
            name: name.into(),
        }
    }

    pub fn new_boxed(ptr: Box<dyn ExternalFn>, name: impl Into<String>) -> Self {
        Self {
            ptr: Locked::new(ptr),
            name: name.into(),
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn call(&self, args: FnCallArg) -> RuntimeResult<Value> {
        (&mut *self.ptr.get_mut()).call(&self.name, args)
    }
}

impl Display for NativeFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NativeFn({})", self.name)
    }
}

impl Debug for NativeFn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeFn")
            .field("ptr", &"Box(dyn ExternalFn)")
            .field("name", &self.name)
            .finish()
    }
}

impl PartialEq for NativeFn {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}
