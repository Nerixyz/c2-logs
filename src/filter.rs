use std::collections::HashSet;

use widestring::{u16str, U16Str, U16String};

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, clap::ValueEnum)]
pub enum FilterMode {
    Include,
    #[default]
    Exclude,
}

pub struct Filter {
    pub mode: FilterMode,
    pub categories: HashSet<U16String>,
}

impl Filter {
    pub fn should_print(&self, category: &U16Str) -> bool {
        include_category(category, self.mode, &self.categories)
    }
}

/// Tries to extract the log-category (if available).
/// **Example:** For `chatterino.foo`, `foo` is returned.
pub fn get_category(line: &U16Str) -> Option<&U16Str> {
    const PREFIX: &U16Str = u16str!("chatterino.");

    let line = line.as_slice();
    if line.starts_with(PREFIX.as_slice()) && line.len() > PREFIX.len() + 2 {
        let pos = line.iter().position(|&x| x == b':' as u16)?;
        Some(U16Str::from_slice(&line[PREFIX.len()..pos]))
    } else {
        None
    }
}

fn include_category(category: &U16Str, mode: FilterMode, filters: &HashSet<U16String>) -> bool {
    match mode {
        FilterMode::Exclude => !filters.contains(category),
        FilterMode::Include => filters.contains(category),
    }
}

#[cfg(test)]
mod tests {
    mod get_category {

        use crate::filter::*;

        #[test]
        fn has_category() {
            for (line, cat) in [
                (u16str!("chatterino.irc: hey"), u16str!("irc")),
                (u16str!("chatterino.http: hey"), u16str!("http")),
                (u16str!("chatterino.seventv.xd: hey"), u16str!("seventv.xd")),
                (u16str!("chatterino.irc: hey"), u16str!("irc")),
                (u16str!("chatterino.irc:lol"), u16str!("irc")),
                (u16str!("chatterino.:lol"), u16str!("")),
            ] {
                assert_eq!(get_category(line), Some(cat))
            }
        }

        #[test]
        fn no_category() {
            for line in [
                u16str!("chatterino.irc"),
                u16str!("chatterino: hey"),
                u16str!("libpng warning: invalid xd"),
                u16str!("qt.irc: hey"),
                u16str!(""),
            ] {
                assert_eq!(get_category(line), None)
            }
        }
    }

    mod include_category {
        use std::collections::HashSet;

        use crate::filter::*;

        #[test]
        fn include() {
            let filters = HashSet::from_iter(
                [u16str!("irc"), u16str!("http")]
                    .into_iter()
                    .map(ToOwned::to_owned),
            );

            for cat in [u16str!("irc"), u16str!("http")] {
                assert!(include_category(cat, FilterMode::Include, &filters));
            }

            for cat in [
                u16str!("seventv"),
                u16str!("bttv"),
                u16str!("twitch"),
                u16str!(""),
            ] {
                assert!(!include_category(cat, FilterMode::Include, &filters));
            }
        }

        #[test]
        fn exclude() {
            let filters = HashSet::from_iter(
                [u16str!("irc"), u16str!("http")]
                    .into_iter()
                    .map(ToOwned::to_owned),
            );

            for cat in [u16str!("irc"), u16str!("http")] {
                assert!(!include_category(cat, FilterMode::Exclude, &filters));
            }

            for cat in [
                u16str!("seventv"),
                u16str!("bttv"),
                u16str!("twitch"),
                u16str!(""),
            ] {
                assert!(include_category(cat, FilterMode::Exclude, &filters));
            }
        }
    }
}
