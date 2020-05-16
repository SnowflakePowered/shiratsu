use super::{ParseError, Result as DatResult};
use quick_xml::de::{from_reader as from_xml_buf, from_str as from_xml, DeError};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::io::BufRead;

#[derive(Debug, Deserialize, PartialEq)]
struct Header {
    homepage: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(super) struct Datfile<T> {
    pub(super) game: Vec<T>,
    header: Option<Header>,
}

pub(super) fn parse_dat<G: PartialEq + DeserializeOwned, E: Into<ParseError> + From<DeError>>(
    f: &str,
    expect_homepage: Option<&'static str>,
) -> DatResult<Datfile<G>> {
    parse_dat_unchecked::<G, E>(f).and_then(|e| {
        if let (None, &None) = (expect_homepage, &e.header)
        {
           return Ok(e);
        } else if let (None, &Some(&None)) = (expect_homepage, &e.header.as_ref().map(|header| &header.homepage)){
            return Ok(e);
        } else if let Some(expected) = expect_homepage {
            if expected == e.header.as_ref().unwrap().homepage.as_deref().unwrap() {
                return Ok(e);
            }
        }
        Err(ParseError::HeaderMismatchError(
            expect_homepage.unwrap(),
            e.header.map(|h| h.homepage).flatten(),
        ))
    })
}

pub(super) fn parse_dat_unchecked<
    G: PartialEq + DeserializeOwned,
    E: Into<ParseError> + From<DeError>,
>(
    f: &str,
) -> DatResult<Datfile<G>> {
    let d: Result<Datfile<G>, E> = from_xml(f).map_err::<E, _>(|e| e.into());
    d.map_err(|e| e.into())
}

pub(super) fn parse_dat_buf<
    R: BufRead,
    G: PartialEq + DeserializeOwned,
    E: Into<ParseError> + From<DeError>,
>(
    f: R,
    expect_homepage: Option<&'static str>,
) -> DatResult<Datfile<G>> {
    parse_dat_unchecked_buf::<R, G, E>(f).and_then(|e| {
        if let (None, &None) = (expect_homepage, &e.header)
        {
           return Ok(e);
        } else if let (None, &Some(&None)) = (expect_homepage, &e.header.as_ref().map(|header| &header.homepage)){
            return Ok(e);
        } else if let Some(expected) = expect_homepage {
            if let Some(homepage) = &e.header.as_ref().and_then(|header| header.homepage.as_deref()) {
                if expected == *homepage {
                    return Ok(e);
                }
            }
        }
        Err(ParseError::HeaderMismatchError(
            expect_homepage.unwrap(),
            e.header.map(|h| h.homepage).flatten(),
        ))
    })
}

pub(super) fn parse_dat_unchecked_buf<
    R: BufRead,
    G: PartialEq + DeserializeOwned,
    E: Into<ParseError> + From<DeError>,
>(
    f: R,
) -> DatResult<Datfile<G>> {
    let d: Result<Datfile<G>, E> = from_xml_buf(f).map_err::<E, _>(|e| e.into());
    d.map_err(|e| e.into())
}
