use std::collections::HashSet;

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq, clap::ValueEnum)]
pub enum FilterMode {
    Include,
    #[default]
    Exclude,
}

pub struct Filter {
    pub mode: FilterMode,
    pub categories: HashSet<String>,
}

impl Filter {
    pub fn should_print(&self, line: &[u8]) -> bool {
        match get_category(line) {
            Some(cat) => include_category(cat, self.mode, &self.categories),
            None => match self.mode {
                FilterMode::Include => false,
                FilterMode::Exclude => true,
            },
        }
    }
}

fn get_category(line: &[u8]) -> Option<&str> {
    if line.starts_with(b"chatterino.") && line.len() > b"chatterino.".len() + 2 {
        let pos = line.iter().position(|&x| x == b':')?;
        std::str::from_utf8(&line[b"chatterino.".len()..pos]).ok()
    } else {
        None
    }
}

fn include_category(category: &str, mode: FilterMode, filters: &HashSet<String>) -> bool {
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
                (b"chatterino.irc: hey" as &[u8], "irc"),
                (b"chatterino.http: hey", "http"),
                (b"chatterino.seventv.xd: hey", "seventv.xd"),
                (b"chatterino.irc: hey", "irc"),
                (b"chatterino.irc:lol", "irc"),
            ] {
                assert_eq!(get_category(line), Some(cat))
            }
        }

        #[test]
        fn no_category() {
            for line in [
                b"chatterino.irc" as &[u8],
                b"chatterino: hey",
                b"libpng warning: invalid xd",
                b"qt.irc: hey",
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
            let filters = HashSet::from_iter(["irc", "http"].into_iter().map(ToOwned::to_owned));

            for cat in ["irc", "http"] {
                assert!(include_category(cat, FilterMode::Include, &filters));
            }

            for cat in ["seventv", "bttv", "twitch"] {
                assert!(!include_category(cat, FilterMode::Include, &filters));
            }
        }

        #[test]
        fn exclude() {
            let filters = HashSet::from_iter(["irc", "http"].into_iter().map(ToOwned::to_owned));

            for cat in ["irc", "http"] {
                assert!(!include_category(cat, FilterMode::Exclude, &filters));
            }

            for cat in ["seventv", "bttv", "twitch"] {
                assert!(include_category(cat, FilterMode::Exclude, &filters));
            }
        }
    }

    mod should_print {
        use crate::filter::*;

        #[test]
        fn include() {
            let filter = Filter {
                categories: HashSet::from_iter(["irc", "http"].into_iter().map(ToOwned::to_owned)),
                mode: FilterMode::Include,
            };

            for line in [
                b"chatterino.irc: hey" as &[u8],
                b"chatterino.http: hey",
                b"chatterino.irc:lol",
            ] {
                assert!(
                    filter.should_print(line),
                    "[print] line={}",
                    std::str::from_utf8(line).unwrap()
                );
            }

            for line in [
                b"chatterino.seventv: hey" as &[u8],
                b"chatterino.bttv: hey",
                b"chatterino.twitch:lol",
                b"libpng warning: forsen",
                b"qt.irc: hey",
                b"xd",
            ] {
                assert!(
                    !filter.should_print(line),
                    "[no-print] line={}",
                    std::str::from_utf8(line).unwrap()
                );
            }
        }

        #[test]
        fn exclude() {
            let filter = Filter {
                categories: HashSet::from_iter(["irc", "http"].into_iter().map(ToOwned::to_owned)),
                mode: FilterMode::Exclude,
            };

            for line in [
                b"chatterino.seventv: hey" as &[u8],
                b"chatterino.bttv: hey",
                b"chatterino.twitch:lol",
                b"libpng warning: forsen",
                b"qt.irc: hey",
                b"xd",
            ] {
                assert!(
                    filter.should_print(line),
                    "[print] line={}",
                    std::str::from_utf8(line).unwrap()
                );
            }

            for line in [
                b"chatterino.irc: hey" as &[u8],
                b"chatterino.http: hey",
                b"chatterino.irc:lol",
            ] {
                assert!(
                    !filter.should_print(line),
                    "[no-print] line={}",
                    std::str::from_utf8(line).unwrap()
                );
            }
        }
    }
}
