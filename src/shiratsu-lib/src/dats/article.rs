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

pub(super) struct Article(&'static str, &'static str, Regex);

impl Article {
    fn find(&self, text: &str) -> Option<usize> {
        self.2.find(text).map(|m| m.start())
    }
    const fn len_from(&self, idx: usize) -> usize {
        self.0.len() + idx
    }
}
lazy_static! {
    pub(super) static ref ARTICLES: Vec<Article> = vec![
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

pub(super) fn move_article(mut text: String, articles: &[Article]) -> String {
    let min_art = articles
        .iter()
        .filter_map(|art| art.find(&text).map(|idx| (art, idx)))
        .min_by_key(|(_, idx)| *idx);

    match min_art {
        None => text,
        Some((article, index)) => {
            text.replace_range(index..article.len_from(index), "");
            text.insert_str(0, article.1);
            text
        }
    }
}
