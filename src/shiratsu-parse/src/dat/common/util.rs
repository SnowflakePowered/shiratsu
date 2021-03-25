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

        use crate::dat::xml::*;
        use crate::dat::*;

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

//
// /// Provides methods that parse an XML .dat files from [No-Intro](https://datomatic.no-intro.org/)
// pub trait FromNoIntro {
//     /// Parses the contents of a No-Intro XML DAT into a vector of `GameEntries`
//     /// This function will check that the
//     /// XML has the proper header for No-Intro DATs. Use
//     /// `parse_nointro_unchecked` if you wish to ignore the header.
//     fn try_from_nointro_str(dat: &str) -> Result<Vec<Result<GameEntry>>>;
//
//     /// Parses the contents of a No-Intro XML DAT into a vector of `GameEntries`,
//     /// ignoring the header element.
//     fn try_unchecked_from_nointro_str(dat: &str) -> Result<Vec<Result<GameEntry>>>;
//
//     /// Parses the contents of a No-Intro XML DAT into a vector of `GameEntries`
//     /// This function will check that the
//     /// XML has the proper header for No-Intro DATs. Use
//     /// `parse_nointro_unchecked` if you wish to ignore the header.
//     fn try_from_nointro_buf<R: std::io::BufRead>(buf: R) -> Result<Vec<Result<GameEntry>>>;
//
//     /// Parses the contents of a No-Intro XML DAT into a vector of `GameEntries`,
//     /// ignoring the header element
//     fn try_unchecked_from_nointro_buf<R: std::io::BufRead>(
//         buf: R,
//     ) -> Result<Vec<Result<GameEntry>>>;
// }
//
// impl FromNoIntro for GameEntry {
//     fn try_from_nointro_str(dat: &str) -> Result<Vec<Result<GameEntry>>> {
//         parse(dat)
//     }
//     fn try_unchecked_from_nointro_str(dat: &str) -> Result<Vec<Result<GameEntry>>> {
//         parse_unchecked(dat)
//     }
//     fn try_from_nointro_buf<R: std::io::BufRead>(buf: R) -> Result<Vec<Result<GameEntry>>> {
//         parse_buf(buf)
//     }
//     fn try_unchecked_from_nointro_buf<R: std::io::BufRead>(
//         buf: R,
//     ) -> Result<Vec<Result<GameEntry>>> {
//         parse_unchecked_buf(buf)
//     }
// }
//

macro_rules! make_from {
    ($hp: expr, $url: expr, $upper: ident, $lower: ident) => {
        use paste::paste;

        use crate::dat::GameEntry;

        paste! {
            /// Provides methods that parse XML .dat files from [
            #[doc=$hp]
            ///](
            #[doc=$url]
            ///)
            pub trait [<From $upper>] {
                /// Parses the contents of a
                #[doc=$hp]
                /// XML DAT into a vector of `GameEntries`
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
                /// XML DAT into a vector of `GameEntries`
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
