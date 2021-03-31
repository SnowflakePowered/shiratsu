use indexmap::IndexSet;
use phf::phf_map;
use std::array;

/// Region parsing errors.
#[derive(Debug, PartialEq)]
pub enum RegionError {
    /// A parsing error occurred.
    ///
    /// The format was invalid for `(RegionFormat, index, column)`
    BadRegionCode(RegionFormat, usize, usize),

    /// No regions were found during parsing.
    NoRegions(RegionFormat),
}

impl std::error::Error for RegionError {}

impl std::fmt::Display for RegionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

type Result<T> = std::result::Result<T, RegionError>;

/// Possible region conventions.
#[derive(Debug, Eq, PartialEq)]
pub enum RegionFormat {
    /// TOSEC naming standards (using appended ISO country codes)
    TOSEC,
    /// GoodTools naming standards ((U), (J), etc.)
    GoodTools,
    /// No-Intro naming standards (country names)
    NoIntro,
}

static TOSEC_REGION: phf::Map<&'static str, Region> = phf_map! {
    "AE" => Region::UnitedArabEmirates,
    "AL" => Region::Albania,
    "AR" => Region::Argentina,
    "AS" => Region::Asia,
    "AT" => Region::Austria,
    "AU" => Region::Australia,
    "BA" => Region::Bosnia,
    "BE" => Region::Belgium,
    "BG" => Region::Bulgaria,
    "BR" => Region::Brazil,
    "CA" => Region::Canada,
    "CH" => Region::Switzerland,
    "CL" => Region::Chile,
    "CN" => Region::China,
    "CS" => Region::Serbia,
    "CY" => Region::Cyprus,
    "CZ" => Region::Czechia,
    "DE" => Region::Germany,
    "DK" => Region::Denmark,
    "EE" => Region::Estonia,
    "EG" => Region::Egypt,
    "ES" => Region::Spain,
    "EU" => Region::Europe,
    "FI" => Region::Finland,
    "FR" => Region::France,
    "GB" => Region::UnitedKingdom,
    "GR" => Region::Greece,
    "HK" => Region::HongKong,
    "HR" => Region::Croatia,
    "HU" => Region::Hungary,
    "ID" => Region::Indonesia,
    "IE" => Region::Ireland,
    "IL" => Region::Israel,
    "IN" => Region::India,
    "IR" => Region::Iran,
    "IS" => Region::Iceland,
    "IT" => Region::Italy,
    "JO" => Region::Jordan,
    "JP" => Region::Japan,
    "KR" => Region::SouthKorea,
    "LT" => Region::Lithuania,
    "LU" => Region::Luxembourg,
    "LV" => Region::Latvia,
    "MN" => Region::Mongolia,
    "MX" => Region::Mexico,
    "MY" => Region::Malaysia,
    "NL" => Region::Netherlands,
    "NO" => Region::Norway,
    "NP" => Region::Nepal,
    "NZ" => Region::NewZealand,
    "OM" => Region::Oman,
    "PE" => Region::Peru,
    "PH" => Region::Philippines,
    "PL" => Region::Poland,
    "PT" => Region::Portugal,
    "QA" => Region::Qatar,
    "RO" => Region::Romania,
    "RU" => Region::Russia,
    "SE" => Region::Sweden,
    "SG" => Region::Singapore,
    "SI" => Region::Slovenia,
    "SK" => Region::Slovakia,
    "TH" => Region::Thailand,
    "TR" => Region::Turkey,
    "TW" => Region::Taiwan,
    "US" => Region::UnitedStates,
    "VN" => Region::Vietnam,
    "YU" => Region::Yugoslavia,
    "ZA" => Region::SouthAfrica,
    "ZZ" => Region::Unknown,
};

// GoodCodes.txt
// also https://segaretro.org/GoodTools
static GOODTOOLS_REGION: phf::Map<&'static str, Region> = phf_map! {
    "A" => Region::Australia,
    "As" => Region::Asia,
    "B" => Region::Brazil,
    "C" => Region::Canada,
    "Ch" => Region::China,
    "Cz" => Region::Czechia,    // Does not exist in 2016-4-3, might exist as a TOSEC code?
    "D" => Region::Netherlands, // 'D' for Dutch
    "E" => Region::Europe,
    "F" => Region::France,
    "G" => Region::Germany,
    "Gr" => Region::Greece,
    "HK" => Region::HongKong,
    "I" => Region::Italy,
    "J" => Region::Japan,
    "K" => Region::SouthKorea,
    "Nl" => Region::Netherlands,
    "No" => Region::Norway,
    "R" => Region::Russia,
    "S" => Region::Spain,
    "Sw" => Region::Sweden,
    "U" => Region::UnitedStates,
    "UK" => Region::UnitedKingdom,
    "Unk" => Region::Unknown,
};

static NOINTRO_REGION: phf::Map<&'static str, Region> = phf_map! {
    "Australia" => Region::Australia,
    "Argentina" => Region::Argentina,
    "Brazil" => Region::Brazil,
    "Canada" => Region::Canada,
    "China" => Region::China,
    "Denmark" => Region::Denmark,
    "Netherlands" => Region::Netherlands,
    "Europe" => Region::Europe,
    "France" => Region::France,
    "Germany" => Region::Germany,
    "Greece" => Region::Greece,
    "Hong Kong" => Region::HongKong,
    "Italy" => Region::Italy,
    "Japan" => Region::Japan,
    "Korea" => Region::SouthKorea,
    "Norway" => Region::Norway,
    "Russia" => Region::Russia,
    "Spain" => Region::Spain,
    "Sweden" => Region::Sweden,
    "USA" => Region::UnitedStates,
    "UK" => Region::UnitedKingdom,
    "United Kingdom" => Region::UnitedKingdom,
    "Asia" => Region::Asia,
    "Poland" => Region::Poland,
    "Portugal" => Region::Portugal,
    "Ireland" => Region::Ireland,
    "Unknown" => Region::Unknown,
    "Taiwan" => Region::Taiwan,
    "Finland" => Region::Finland,
    "UAE" => Region::UnitedArabEmirates,
    "Albania" => Region::Albania,
    "Austria" => Region::Austria,
    "Bosnia" => Region::Bosnia,
    "Belgium" => Region::Belgium,
    "Bulgaria" => Region::Bulgaria,
    "Switzerland" => Region::Switzerland,
    "Chile" => Region::Chile,
    "Serbia" => Region::Serbia,
    "Cyprus" => Region::Cyprus,
    "Czech Republic" => Region::Czechia,
    "Czechia" => Region::Czechia,
    "Estonia" => Region::Estonia,
    "Egypt" => Region::Egypt,
    "Croatia" => Region::Croatia,
    "Hungary" => Region::Hungary,
    "Indonesia" => Region::Indonesia,
    "Israel" => Region::Israel,
    "India" => Region::India,
    "Iran" => Region::Iran,
    "Iceland" => Region::Iceland,
    "Jordan" => Region::Jordan,
    "Lithuania" => Region::Lithuania,
    "Luxembourg" => Region::Luxembourg,
    "Latvia" => Region::Latvia,
    "Mongolia" => Region::Mongolia,
    "Mexico" => Region::Mexico,
    "Malaysia" => Region::Malaysia,
    "Nepal" => Region::Nepal,
    "New Zealand" => Region::NewZealand,
    "Oman" => Region::Oman,
    "Peru" => Region::Peru,
    "Philippines" => Region::Philippines,
    "Qatar" => Region::Qatar,
    "Romania" => Region::Romania,
    "Singapore" => Region::Singapore,
    "Slovenia" => Region::Slovenia,
    "Slovakia" => Region::Slovakia,
    "Thailand" => Region::Thailand,
    "Turkey" => Region::Turkey,
    "Vietnam" => Region::Vietnam,
    "Yugoslavia" => Region::Yugoslavia,
    "South Africa" => Region::SouthAfrica,
    "The Netherlands" => Region::Netherlands,
};

/// Possible regions of a ROM file taken mostly from TOSEC and No-Intro
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Ord, PartialOrd)]
pub enum Region {
    Unknown,
    UnitedArabEmirates,
    Albania,
    Asia,
    Argentina,
    Austria,
    Australia,
    Bosnia,
    Belgium,
    Bulgaria,
    Brazil,
    Canada,
    Switzerland,
    Chile,
    China,
    Serbia,
    Cyprus,
    Czechia,
    Germany,
    Denmark,
    Estonia,
    Egypt,
    Spain,
    Europe,
    Finland,
    France,
    UnitedKingdom,
    Greece,
    HongKong,
    Croatia,
    Hungary,
    Indonesia,
    Ireland,
    Israel,
    India,
    Iran,
    Iceland,
    Italy,
    Jordan,
    Japan,
    SouthKorea,
    Lithuania,
    Luxembourg,
    Latvia,
    Mongolia,
    Mexico,
    Malaysia,
    Netherlands,
    Norway,
    Nepal,
    NewZealand,
    Oman,
    Peru,
    Philippines,
    Poland,
    Portugal,
    Qatar,
    Romania,
    Russia,
    Sweden,
    Singapore,
    Slovenia,
    Slovakia,
    Thailand,
    Turkey,
    Taiwan,
    UnitedStates,
    Vietnam,
    Yugoslavia,
    SouthAfrica,
}

impl Region {
    /// Parse a valid TOSEC region string into a `Vec<Region>`.
    /// A valid region string is 2 uppercase letter country codes, separated by hyphens.
    ///
    /// # Arguments
    /// - `region_str` The region string.
    pub fn try_from_tosec_region<S: AsRef<str> + ?Sized>(region_str: &S) -> Result<Vec<Self>> {
        let (_, region) = from_tosec_region(region_str)?;
        Ok(region)
    }

    /// Parse a valid TOSEC region string into a vector including the
    /// strings corresponding to each parsed region.
    ///
    /// A valid region string is 2 uppercase letter country codes, separated by hyphens.
    ///
    /// # Arguments
    /// - `region_str` The region string.
    pub fn try_from_tosec_region_with_strs<S: AsRef<str> + ?Sized>(region_str: &S) -> Result<(Vec<&str>, Vec<Self>)>  {
        from_tosec_region(region_str)
    }

    /// Parse a valid No-Intro region string into a `Vec<Region>`.
    /// A valid region string is a comma + space separated list of valid country names.
    ///
    /// Country names are case sensitive.
    ///
    /// The following strings are expanded
    ///
    /// - `World` is expanded to USA, Japan, and Europe.
    /// - `Scandinavia` is expanded to Denmark, Norway, and Sweden.
    /// - `Latin America` is expanded to Mexico, Brazil, Argentina, Chile, and Peru
    /// # Arguments
    /// - `region_str` The region string.
    pub fn try_from_nointro_region<S: AsRef<str> + ?Sized>(region_str: &S) -> Result<Vec<Self>> {
        let (_, region) = from_nointro_region(region_str)?;
        Ok(region)
    }

    /// Parse a valid No-Intro region string into a vector including the
    /// strings corresponding to each parsed region.
    /// A valid region string is a comma + space separated list of valid country names.
    ///
    /// Country names are case sensitive.
    ///
    /// The following strings are expanded
    ///
    /// - `World` is expanded to USA, Japan, and Europe.
    /// - `Scandinavia` is expanded to Denmark, Norway, and Sweden.
    /// - `Latin America` is expanded to Mexico, Brazil, Argentina, Chile, and Peru
    /// # Arguments
    /// - `region_str` The region string.
    pub fn try_from_nointro_region_with_strs<S: AsRef<str> + ?Sized>(region_str: &S) -> Result<(Vec<&str>, Vec<Self>)> {
        from_nointro_region(region_str)
    }

    /// Parse a valid GoodTools region string into a `Vec<Region>`.
    ///
    /// # Arguments
    /// - `region_str` The region string.
    pub fn try_from_goodtools_region<S: AsRef<str> + ?Sized>(region_str: &S) -> Result<Vec<Self>> {
        let (_, region) = from_goodtools_region(region_str)?;
        Ok(region)
    }

    /// Parse a valid GoodTools region string into a vector including the
    /// strings corresponding to each parsed region.
    /// # Arguments
    /// - `region_str` The region string.
    pub fn try_from_goodtools_region_with_strs<S: AsRef<str> + ?Sized>(region_str: &S) -> Result<(Vec<&str>, Vec<Self>)> {
        from_goodtools_region(region_str)
    }

    /// Creates a TOSEC-compatible ISO code region string, separated by hyphens,
    /// from a vector of Region.
    pub fn to_normalized_region_string(regions: &[Self]) -> String {
        to_normalized_region_string(regions)
    }

    /// Best-guess a region string from one of the three known formats.
    /// Returns the format that matches the best (meaning it contains the longest number of matches, excluding 'Unknown')
    ///
    /// This function expects that the input string is a valid GoodTools, No-Intro, or TOSEC region string.
    /// If no match can be found, returns unknown region.
    pub fn from_region_string<T: AsRef<str>>(region_str: T) -> Vec<Self> {
        parse_regions(region_str)
    }
}

/// Parse a valid TOSEC region string into a `Vec<Region>`.
/// A valid region string is 2 uppercase letter country codes, separated by hyphens.
///
/// # Arguments
/// - `region_str` The region string.
fn from_tosec_region<S: AsRef<str> + ?Sized>(region_str: &S) -> Result<(Vec<&str>, Vec<Region>)> {
    let mut region_strings = Vec::new();
    let mut regions: IndexSet<Region> = IndexSet::new();
    let mut region_count = 0;
    let mut region_string_index = 0;

    for region_code in region_str.as_ref().split('-')
    {
        let region = TOSEC_REGION.get(region_code);
        if region_code.len() != 2 {
            return Err(RegionError::BadRegionCode(RegionFormat::TOSEC,
                                                  region_count,
                                                  region_string_index))
        }
        if let Some(region) = region {
            regions.insert(*region);
            region_strings.push(region_code);
            region_count += 1;
            region_string_index += region_code.len();
            region_string_index += "-".len();
        } else {
            return Err(RegionError::BadRegionCode(RegionFormat::TOSEC,
                                                  region_count,
                                                  region_string_index))
        }
    }

    if regions.is_empty() {
         Err(RegionError::NoRegions(RegionFormat::TOSEC))
    } else {
        Ok((region_strings, regions.into_iter().collect::<Vec<Region>>()))
    }
}

/// Parse a valid GoodTools region string into a `Vec<Region>`.
///
/// # Arguments
/// - `region_str` The region string.
fn from_goodtools_region<S: AsRef<str> + ?Sized>(region_str: &S) -> Result<(Vec<&str>, Vec<Region>)> {
    let mut regions = IndexSet::<Region>::new();
    let mut region_strings = Vec::new();

    let mut region_count = 0;
    let mut region_string_index = 0;

    for region_code in region_str.as_ref().split(",") {
        match region_code {
            "1" => {
                regions.insert(Region::Japan);
                regions.insert(Region::SouthKorea);
            }
            "4" => {
                regions.insert(Region::UnitedStates);
                regions.insert(Region::Brazil);
            }
            "5" => {
                regions.insert(Region::Japan);
                regions.insert(Region::UnitedStates);
            }
            "W" | "JUE" => {
                regions.insert(Region::Japan);
                regions.insert(Region::UnitedStates);
                regions.insert(Region::Europe);
            }
            "UE" => {
                regions.insert(Region::UnitedStates);
                regions.insert(Region::Europe);
            }
            "JU" => {
                regions.insert(Region::Japan);
                regions.insert(Region::UnitedStates);
            }
            _ => match GOODTOOLS_REGION.get(region_code)
            {
                Some(&region) => {
                    regions.insert(region);
                }
                None => return Err(
                    RegionError::BadRegionCode(RegionFormat::GoodTools,
                                               region_count, region_string_index)
                )
            }
        }

        region_strings.push(region_code);
        // If there was error we would have deleted..
        region_count += 1;
        region_string_index += region_code.len();
        region_string_index += ", ".len();
    }

    // invariant: Do not return RegionError::BadRegionCode because region_string_index will
    // overflow.
    if regions.is_empty() {
        Err(RegionError::NoRegions(RegionFormat::GoodTools))
    } else {
        Ok((region_strings, regions.into_iter().collect::<Vec<Region>>()))
    }
}

/// Parse a valid No-Intro region string into a `Vec<Region>`.
/// A valid region string is a comma + space separated list of valid country names.
///
/// Country names are case sensitive.
///
/// The following strings are expanded
///
/// - `World` is expanded to USA, Japan, and Europe.
/// - `Scandinavia` is expanded to Denmark, Norway, and Sweden.
/// # Arguments
/// - `region_str` The region string.
fn from_nointro_region<S: AsRef<str> + ?Sized>(region_str: &S)-> Result<(Vec<&str>, Vec<Region>)> {
    let mut regions = IndexSet::<Region>::new();
    let mut region_strings = Vec::new();

    let mut region_count = 0;
    let mut region_string_index = 0;

    for region_code in region_str.as_ref().split(", ") {
        if !region_code.chars().all(|c| char::is_ascii_alphabetic(&c) || c == ' ') {
            return Err(RegionError::BadRegionCode(
                RegionFormat::NoIntro,
                region_count,
                region_string_index,
            ));
        }

        match region_code {
            "World" | "World (guessed)" | "World (Guessed)" => {
                regions.insert(Region::UnitedStates);
                regions.insert(Region::Japan);
                regions.insert(Region::Europe);
            }
            "Scandinavia" => {
                regions.insert(Region::Denmark);
                regions.insert(Region::Norway);
                regions.insert(Region::Sweden);
            }
            "Latin America" => {
                regions.insert(Region::Mexico);
                regions.insert(Region::Brazil);
                regions.insert(Region::Argentina);
                regions.insert(Region::Chile);
                regions.insert(Region::Peru);
            }
            _ => match NOINTRO_REGION.get(region_code) {
                Some(&region) => {
                    regions.insert(region);
                }
                None => return Err(
                    RegionError::BadRegionCode(RegionFormat::NoIntro,
                                               region_count, region_string_index)
                ),
            },
        }
        region_strings.push(region_code);
        // If there was error we would have deleted..
        region_count += 1;
        region_string_index += region_code.len();
        region_string_index += ", ".len();
    }

    // invariant: Do not return RegionError::BadRegionCode because region_string_index will
    // overflow.
    if regions.is_empty() {
        Err(RegionError::NoRegions(RegionFormat::NoIntro))
    } else {
        Ok((region_strings, regions.into_iter().collect::<Vec<Region>>()))
    }
}

/// Creates a TOSEC-compatible ISO code region string, separated by hyphens,
/// from a vector of Region.
fn to_normalized_region_string(regions: &[Region]) -> String {
    regions
        .iter()
        .map(|r| r.into())
        .collect::<Vec<&str>>()
        .join("-")
}

/// Best-guess a region string from one of the three known formats.
/// Returns the format that matches the best (meaning it contains the longest number of matches, excluding 'Unknown')
/// This function expects that the input string is a valid GoodTools, No-Intro, or TOSEC region string.
/// If no match can be found, returns unknown region.
fn parse_regions<T: AsRef<str>>(region_str: T) -> Vec<Region> {
    let good_tools_try = from_goodtools_region(&region_str.as_ref())
        .map(|(_, res)| res)
        .unwrap_or(vec![Region::Unknown]);
    let nointro_try = from_nointro_region(&region_str.as_ref())
        .map(|(_, res)| res)
        .unwrap_or(vec![Region::Unknown]);
    let tosec_try = from_tosec_region(&region_str.as_ref())
        .map(|(_, res)| res)
        .unwrap_or(vec![Region::Unknown]);
    // thanks @Rantanen on the Rust discord
    array::IntoIter::new([good_tools_try, nointro_try, tosec_try])
        // Precalculate all the counts so they don't need to be calculated for
        // every single comparison.
        .map(|v| (v.iter().filter(|&r| *r != Region::Unknown).count(), v))
        // Use the count as the key to get max by.
        .max_by_key(|(count, _)| *count)
        // Map the (count, vec) tuple back to the vec.
        // The count has served its purpose.
        .map(|(_, v)| v)
        // In case the option was none (the input Vec was empty), return empty vec.
        .unwrap_or_else(|| vec![Region::Unknown])
}

impl AsRef<str> for Region {
    fn as_ref(&self) -> &str {
        self.into()
    } 
}

impl From<&Region> for &str {
    fn from(region: &Region) -> Self {
        match region {
            Region::UnitedArabEmirates => "AE",
            Region::Albania => "AL",
            Region::Argentina => "AR",
            Region::Asia => "AS",
            Region::Austria => "AT",
            Region::Australia => "AU",
            Region::Bosnia => "BA",
            Region::Belgium => "BE",
            Region::Bulgaria => "BG",
            Region::Brazil => "BR",
            Region::Canada => "CA",
            Region::Switzerland => "CH",
            Region::Chile => "CL",
            Region::China => "CN",
            Region::Serbia => "CS",
            Region::Cyprus => "CY",
            Region::Czechia => "CZ",
            Region::Germany => "DE",
            Region::Denmark => "DK",
            Region::Estonia => "EE",
            Region::Egypt => "EG",
            Region::Spain => "ES",
            Region::Europe => "EU",
            Region::Finland => "FI",
            Region::France => "FR",
            Region::UnitedKingdom => "GB",
            Region::Greece => "GR",
            Region::HongKong => "HK",
            Region::Croatia => "HR",
            Region::Hungary => "HU",
            Region::Indonesia => "ID",
            Region::Ireland => "IE",
            Region::Israel => "IL",
            Region::India => "IN",
            Region::Iran => "IR",
            Region::Iceland => "IS",
            Region::Italy => "IT",
            Region::Jordan => "JO",
            Region::Japan => "JP",
            Region::SouthKorea => "KR",
            Region::Lithuania => "LT",
            Region::Luxembourg => "LU",
            Region::Latvia => "LV",
            Region::Mongolia => "MN",
            Region::Mexico => "MX",
            Region::Malaysia => "MY",
            Region::Netherlands => "NL",
            Region::Norway => "NO",
            Region::Nepal => "NP",
            Region::NewZealand => "NZ",
            Region::Oman => "OM",
            Region::Peru => "PE",
            Region::Philippines => "PH",
            Region::Poland => "PL",
            Region::Portugal => "PT",
            Region::Qatar => "QA",
            Region::Romania => "RO",
            Region::Russia => "RU",
            Region::Sweden => "SE",
            Region::Singapore => "SG",
            Region::Slovenia => "SI",
            Region::Slovakia => "SK",
            Region::Thailand => "TH",
            Region::Turkey => "TR",
            Region::Taiwan => "TW",
            Region::UnitedStates => "US",
            Region::Vietnam => "VN",
            Region::Yugoslavia => "YU",
            Region::SouthAfrica => "ZA",
            Region::Unknown => "ZZ",
        }
    }
}
