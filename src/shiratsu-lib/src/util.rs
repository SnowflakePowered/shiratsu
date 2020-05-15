// todo: export this out to its own library.
// thanks @Rantanen on the Rust discord for the idea.

#[macro_export]
macro_rules! chained_iter {
    ($elem: expr) => (
        once($elem)
    );
    ($first: expr, $($rest: expr), + $(,)?) => {
        once($first)$(.chain(once($rest)))*
    }
}

#[macro_export(local_inner_macros)]
macro_rules! wrap_error {
    ($(wrap $wrapper:ident ($error: ty) for $coerce: ty { fn from ($err: ident) { $body: expr } })*) => {
        $(
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
    ($(wrap <$type:tt> $wrapper:ident ($error: ty) for $coerce: ty { fn from ($err: ident) { $body: expr } })*) => {
        $(
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