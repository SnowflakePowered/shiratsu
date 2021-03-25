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
    use crate::naming::util::replace_hyphen_mut;

    #[test]
    fn test_replace_hyphen()
    {
        let mut s = String::from("Hello - World - Foo - Bar");
        replace_hyphen_mut(&mut s);
        assert_eq!(s, String::from("Hello: World: Foo: Bar"));
    }
}