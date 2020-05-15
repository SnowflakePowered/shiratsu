use super::{DatError, Result as DatResult};
use quick_xml::de::{from_str as from_xml, from_reader as from_xml_buf, DeError};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::io::BufRead;

#[derive(Debug, Deserialize, PartialEq)]
struct Header {
    pub(super) homepage: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
pub(super) struct Datfile<T> {
    pub(super) game: Vec<T>,
    header: Option<Header>,
}

pub(super) fn parse_dat<G: PartialEq + DeserializeOwned, E: Into<DatError> + From<DeError>>(
    f: &str,
    expect_homepage: Option<&'static str>,
) -> DatResult<Datfile<G>> {
    parse_dat_unchecked::<G, E>(f).and_then(|e| {
        if expect_homepage.is_none() && e.header.is_none() || expect_homepage.is_none() 
            && e.header.as_ref().unwrap().homepage.is_none() {
            return Ok(e);
        } else if let Some(expected) = expect_homepage {
            if expected == e.header.as_ref().unwrap().homepage.as_deref().unwrap() {
                return Ok(e);
            }
        }
        Err(DatError::HeaderMismatchError(
            expect_homepage.unwrap(),
            e.header.map(|h| h.homepage).flatten(),
        ))
    })
}

pub(super) fn parse_dat_unchecked<
    G: PartialEq + DeserializeOwned,
    E: Into<DatError> + From<DeError>,
>(
    f: &str,
) -> DatResult<Datfile<G>> {
    let d: Result<Datfile<G>, E> = from_xml(f).map_err::<E, _>(|e| e.into());
    d.map_err(|e| e.into())
}

pub(super) fn parse_dat_buf<R: BufRead, G: PartialEq + DeserializeOwned, E: Into<DatError> + From<DeError>>(
    f: R,
    expect_homepage: Option<&'static str>,
) -> DatResult<Datfile<G>> {
    parse_dat_unchecked_buf::<R, G, E>(f).and_then(|e| {
        if expect_homepage.is_none() && e.header.is_none() || expect_homepage.is_none() 
            && e.header.as_ref().unwrap().homepage.is_none() {
            return Ok(e);
        } else if let Some(expected) = expect_homepage {
            if expected == e.header.as_ref().unwrap().homepage.as_deref().unwrap() {
                return Ok(e);
            }
        }
        Err(DatError::HeaderMismatchError(
            expect_homepage.unwrap(),
            e.header.map(|h| h.homepage).flatten(),
        ))
    })
}

pub(super) fn parse_dat_unchecked_buf<
    R: BufRead,
    G: PartialEq + DeserializeOwned,
    E: Into<DatError> + From<DeError>,
>(
    f: R,
) -> DatResult<Datfile<G>> {
    let d: Result<Datfile<G>, E> = from_xml_buf(f).map_err::<E, _>(|e| e.into());
    d.map_err(|e| e.into())
}

