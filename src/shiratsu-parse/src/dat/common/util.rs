macro_rules! wrap_error {
    ($(wrap $wrapper:ident ($error: ty) for $coerce: ty { fn from ($err: tt) { $body: expr } })*) => {
        $(
            #[derive(Debug)]
            struct $wrapper($error);

            impl From<$error> for $wrapper {
                fn from(err: $error) -> Self {
                    $wrapper(err)
                }
            }

            impl From<$wrapper> for $coerce {
                fn from($err: $wrapper) -> Self
                {
                    $body
                }
            }
        )*
    };
    ($(wrap <$type:tt> $wrapper:ident ($error: ty) for $coerce: ty { fn from ($err: tt) { $body: expr } })*) => {
        $(
            #[derive(Debug)]
            struct $wrapper<$type>($error);

            impl <$type> From<$error> for $wrapper<$type> {
                fn from(err: $error) -> Self {
                    $wrapper(err)
                }
            }

            impl <$type> From<$wrapper<$type>> for $coerce {
                fn from($err: $wrapper<$type>) -> Self
                {
                    $body
                }
            }
        )*
    };
}