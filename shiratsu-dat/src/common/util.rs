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

macro_rules! make_parse {
    ($hp: literal, $game: ty, $error: ty) => {

        use crate::xml::*;
        use crate::*;

        fn parse(f: &str) -> Result<Vec<Result<GameEntry>>> {
            Ok(parse_dat::<$game, $error>(f, Some($hp))?
                .game
                .into_iter()
                .map(|g| g.try_into())
                .collect())
        }

        fn parse_unchecked(f: &str) -> Result<Vec<Result<GameEntry>>> {
            Ok(parse_dat_unchecked::<$game, $error>(f)?
                .game
                .into_iter()
                .map(|g| g.try_into())
                .collect())
        }

        fn parse_buf<R: std::io::BufRead>(f: R) -> Result<Vec<Result<GameEntry>>> {
            Ok(
                parse_dat_buf::<R, $game, $error>(f, Some($hp))?
                    .game
                    .into_iter()
                    .map(|g| g.try_into())
                    .collect(),
            )
        }
        fn parse_unchecked_buf<R: std::io::BufRead>(f: R) -> Result<Vec<Result<GameEntry>>> {
            Ok(parse_dat_unchecked_buf::<R, $game, $error>(f)?
                .game
                .into_iter()
                .map(|g| g.try_into())
                .collect())
        }
    }
}

macro_rules! make_from {
    ($hp: expr, $url: expr, $upper: ident, $lower: ident) => {
        use paste::paste;

        use crate::GameEntry;

        paste! {
            /// Provides methods that parse XML .dat files from [
            #[doc=$hp]
            ///](
            #[doc=$url]
            ///)
            pub trait [<From $upper>] {
                /// Parses the contents of a
                #[doc=$hp]
                /// XML DAT into a vector of `GameEntries`.
                /// This function will check that the
                /// XML has the proper header for
                #[doc=$hp]
                /// DATs. Use the unchecked variant if you wish to ignore the header.
                fn [<try_from_ $lower _str>](dat: &str) -> Result<Vec<Result<GameEntry>>>;

                /// Parses the contents of a
                #[doc=$hp]
                /// XML DAT into a vector of `GameEntries`,
                /// ignoring the header element.
                fn [<try_unchecked_from_ $lower _str>](dat: &str) -> Result<Vec<Result<GameEntry>>>;

                /// Parses the contents of a
                #[doc=$hp]
                /// XML DAT into a vector of `GameEntries`.
                /// This function will check that the
                /// XML has the proper header for
                #[doc=$hp]
                /// DATs. Use the unchecked variant if you wish to ignore the header.
                fn [<try_from_ $lower _buf>]<R: std::io::BufRead>(buf: R) -> Result<Vec<Result<GameEntry>>>;

                /// Parses the contents of a
                #[doc=$hp]
                /// XML DAT into a vector of `GameEntries`,
                /// ignoring the header element
                fn [<try_unchecked_from_ $lower _buf>]<R: std::io::BufRead>(
                    buf: R,
                ) -> Result<Vec<Result<GameEntry>>>;
            }

            impl [<From $upper>] for GameEntry {
                fn [<try_from_ $lower _str>](dat: &str) -> Result<Vec<Result<GameEntry>>> {
                    parse(dat)
                }
                fn [<try_unchecked_from_ $lower _str>](dat: &str) -> Result<Vec<Result<GameEntry>>> {
                    parse_unchecked(dat)
                }
                fn [<try_from_ $lower _buf>]<R: std::io::BufRead>(buf: R) -> Result<Vec<Result<GameEntry>>> {
                    parse_buf(buf)
                }
                fn [<try_unchecked_from_ $lower _buf>]<R: std::io::BufRead>(
                    buf: R,
                ) -> Result<Vec<Result<GameEntry>>> {
                    parse_unchecked_buf(buf)
                }
            }
        }
    }
}

use lazy_static::lazy_static;
use regex::Regex;

macro_rules! article {
    ($article: expr) => {
        Article(
            concat!(", ", $article),
            concat!($article, " "),
            Regex::new(concat!(", ", $article, "($|\\s)")).unwrap(),
        )
    };
}

pub(crate) struct Article(&'static str, &'static str, Regex);

impl Article {
    fn find(&self, text: &str) -> Option<usize> {
        self.2.find(text).map(|m| m.start())
    }
    const fn len_from(&self, idx: usize) -> usize {
        self.0.len() + idx
    }
}
lazy_static! {
    static ref ARTICLES: Vec<Article> = vec![
        article!("Eine"),
        article!("The"),
        article!("Der"),
        article!("Die"),
        article!("Das"),
        article!("Ein"),
        article!("Les"),
        article!("Los"),
        article!("Las"),
        article!("An"),
        article!("De"),
        article!("La"),
        article!("Le"),
        article!("El"),
        article!("A")
    ];
}

/// From a provided list of articles, mutates the provided title
/// so that the first article encountered comes at the beginning of the string, if
/// it is somewhere after a comma.
///
/// # Arguments
/// - `title`: The string to move
/// - `article`: The articles to check. The first article encountered in the correct position will be moved.
fn move_articles(title: &mut String, articles: &[Article]) {
    let min_art = articles
        .iter()
        .filter_map(|art| art.find(&title).map(|idx| (art, idx)))
        .min_by_key(|(_, idx)| *idx);
    if let Some((article, index)) = min_art {
        title.replace_range(index..article.len_from(index), "");
        title.insert_str(0, article.1);
    }
}


/// Mutates the provided title so that the first article encountered
/// comes at the beginning of the string, if it is somewhere after a comma.
///
/// Uses the default articles.
/// # Arguments
/// - `title`: The string to move
#[inline(always)]
pub(crate) fn move_default_articles_mut(title: &mut String) {
    move_articles(title, &ARTICLES);
}

/// Replaces all hyphens found with a colon.
pub(crate) fn replace_hyphen_mut(title: &mut String) {
    let mut hyphen_index = title.find(" - ");
    while let Some(index) = hyphen_index {
        let new_index = " - ".len() + index;
        title.replace_range(index..new_index, ": ");
        hyphen_index = (&title[new_index..]).find(" - ").map(|f| f + new_index);
    }
}

#[cfg(test)]
mod tests
{
    use crate::common::util::replace_hyphen_mut;

    #[test]
    fn test_replace_hyphen()
    {
        let mut s = String::from("Hello - World - Foo - Bar");
        replace_hyphen_mut(&mut s);
        assert_eq!(s, String::from("Hello: World: Foo: Bar"));
    }
}