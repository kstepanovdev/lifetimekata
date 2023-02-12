use std::{cmp, result};

use require_lifetimes::require_lifetimes;

#[derive(Debug, PartialEq, Eq)]
enum MatcherToken<'a> {
    /// This is just text without anything special.
    RawText(&'a str),
    /// This is when text could be any one of multiple
    /// strings. It looks like `(one|two|three)`, where
    /// `one`, `two` or `three` are the allowed strings.
    OneOfText(Vec<&'a str>),
    /// This is when you're happy to accept any single character.
    /// It looks like `.`
    WildCard,
}

#[derive(Debug, PartialEq, Eq)]
struct Matcher<'a> {
    /// This is the actual text of the matcher
    text: &'a str,
    /// This is a vector of the tokens inside the expression.
    tokens: Vec<MatcherToken<'a>>,
    /// This keeps track of the most tokens that this matcher has matched.
    most_tokens_matched: usize,
}

impl<'a> Matcher<'a> {
    /// This should take a string reference, and return
    /// an `Matcher` which has parsed that reference.
    #[require_lifetimes]
    fn new(text: &'a str) -> Option<Matcher<'a>> {
        let mut tokens = vec![];
        let mut leftovers = text;

        while leftovers.len() > 0 {
            if leftovers.starts_with(".") {
                tokens.push(MatcherToken::WildCard);
                leftovers = &leftovers[1..];
            } else if leftovers.starts_with("(") {
                let Some(close_index) = leftovers.find(")") else { return None };

                let micro_tokens =
                    MatcherToken::OneOfText(leftovers[1..close_index].split("|").collect());
                tokens.push(micro_tokens);

                if (close_index + 1) < leftovers.len() {
                    leftovers = &leftovers[close_index + 1..];
                } else {
                    break;
                }
            } else {
                let next_separator = match (leftovers.find("."), leftovers.find("(")) {
                    (Some(a), Some(b)) => Some(cmp::min(a, b)),
                    (None, Some(a)) | (Some(a), None) => Some(a),
                    (None, None) => None,
                };

                if let Some(index) = next_separator {
                    tokens.push(MatcherToken::RawText(&leftovers[..index]));
                    leftovers = &leftovers[index..];
                } else {
                    tokens.push(MatcherToken::RawText(&leftovers))
                }
            }
        }

        Some(Matcher {
            text,
            tokens,
            most_tokens_matched: 0,
        })
    }

    /// This should take a string, and return a vector of tokens, and the corresponding part
    /// of the given string. For examples, see the test cases below.
    #[require_lifetimes]
    fn match_string<'b, 'c>(&'b mut self, string: &'c str) -> Vec<(&'b MatcherToken<'a>, &'c str)> {
        let mut answer = vec![];
        let mut substring = string;

        'outer_loop: for token in &self.tokens {
            match token {
                MatcherToken::RawText(raw_text) => {
                    if substring.starts_with(raw_text) {
                        answer.push((token, &substring[..raw_text.len()]));
                        substring = &substring[raw_text.len()..];
                    } else {
                        break;
                    }
                }
                MatcherToken::OneOfText(variants) => {
                    for variant in variants {
                        if substring.starts_with(variant) {
                            answer.push((token, &substring[..variant.len()]));
                            substring = &substring[variant.len()..];
                            continue 'outer_loop;
                        }
                    }
                    break;
                }
                MatcherToken::WildCard => {
                    answer.push((token, &substring[..1]));
                    substring = &substring[1..];
                }
            }
        }
        self.most_tokens_matched = answer.len();

        answer
    }
}

fn main() {
    unimplemented!()
}

#[cfg(test)]
mod test {
    use super::{Matcher, MatcherToken};
    #[test]
    fn simple_test() {
        let match_string = "abc(d|e|f).".to_string();
        let mut matcher = Matcher::new(&match_string).unwrap();

        assert_eq!(matcher.most_tokens_matched, 0);

        {
            let candidate1 = "abcge".to_string();
            let result = matcher.match_string(&candidate1);
            assert_eq!(result, vec![(&MatcherToken::RawText("abc"), "abc"),]);
            assert_eq!(matcher.most_tokens_matched, 1);
        }

        {
            // Change 'e' to '💪' if you want to test unicode.
            let candidate1 = "abcde".to_string();
            let result = matcher.match_string(&candidate1);
            assert_eq!(
                result,
                vec![
                    (&MatcherToken::RawText("abc"), "abc"),
                    (&MatcherToken::OneOfText(vec!["d", "e", "f"]), "d"),
                    (&MatcherToken::WildCard, "e")
                ]
            );
            assert_eq!(matcher.most_tokens_matched, 3);
        }
    }

    #[test]
    fn broken_matcher() {
        let match_string = "abc(d|e|f.".to_string();
        let matcher = Matcher::new(&match_string);
        assert_eq!(matcher, None);
    }
}
