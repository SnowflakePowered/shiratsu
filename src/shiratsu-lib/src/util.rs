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
