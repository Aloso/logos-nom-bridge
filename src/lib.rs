//! # logos-nom-bridge
//!
//! A [`logos::Lexer`] wrapper than can be used as an input for
//! [nom](https://docs.rs/nom/7.0.0/nom/index.html).
//!
//! ### Simple example
//!
//! ```
//! // First, create a `logos` lexer:
//!
//! #[derive(Clone, Debug, PartialEq, Eq, logos::Logos)]
//! enum Token {
//!     #[token("+")]
//!     Plus,
//!
//!     #[token("-")]
//!     Minus,
//!
//!     #[regex(r"-?[0-9]+", |lex| lex.slice().parse())]
//!     Number(i64),
//!
//!     #[error]
//!     #[regex(r"[ \t\n\f]+", logos::skip)]
//!     Error,
//! }
//!
//! // Then, write a nom parser that accepts a `Tokens<'_, Token>` as input:
//!
//! use logos_nom_bridge::Tokens;
//!
//! type Input<'source> = Tokens<'source, Token>;
//!
//! #[derive(Debug, PartialEq, Eq)]
//! enum Op {
//!     Number(i64),
//!     Addition(Box<(Op, Op)>),
//!     Subtraction(Box<(Op, Op)>),
//! }
//!
//! fn parse_expression(input: Input<'_>) -> nom::IResult<Input<'_>, Op> {
//! #   use nom::{branch::alt, combinator::map, sequence::tuple};
//! #
//! #   fn parse_number(input: Input<'_>) -> nom::IResult<Input<'_>, Op> {
//! #       match input.peek() {
//! #           Some((Token::Number(n), _)) => Ok((input.advance(), Op::Number(n))),
//! #           _ => Err(nom::Err::Error(nom::error::Error::new(
//! #               input,
//! #               nom::error::ErrorKind::IsA,
//! #           ))),
//! #       }
//! #   }
//! #   logos_nom_bridge::token_parser!(token: Token);
//! #
//! #   alt((
//! #       map(
//! #           tuple((parse_number, alt((Token::Plus, Token::Minus)), parse_expression)),
//! #           |(a, op, b)| {
//! #               if op == "+" {
//! #                   Op::Addition(Box::new((a, b)))
//! #               } else {
//! #                   Op::Subtraction(Box::new((a, b)))
//! #               }
//! #           },
//! #       ),
//! #       parse_number,
//! #   ))(input)
//!     // zip
//! }
//!
//! // Finally, you can use it to parse a string:
//!
//! let input = "10 + 3 - 4";
//! let tokens = Tokens::new(input);
//!
//! let (rest, parsed) = parse_expression(tokens).unwrap();
//!
//! assert!(rest.is_empty());
//! assert_eq!(
//!     parsed,
//!     Op::Addition(Box::new((
//!         Op::Number(10),
//!         Op::Subtraction(Box::new((
//!             Op::Number(3),
//!             Op::Number(4),
//!         ))),
//!     ))),
//! )
//! ```
//!
//! ## Macros
//!
//! You can implement [`nom::Parser`] for your token type with the [`token_parser`] macro:
//!
//! ```
//! # #[derive(Clone, Debug, PartialEq, Eq, logos::Logos)]
//! # enum Token {
//! #     #[error]
//! #     Error,
//! # }
//! #
//! logos_nom_bridge::token_parser!(token: Token);
//! ```
//!
//! If some enum variants of your token type contain data, you can implement a [`nom::Parser`]
//! for them using the [`data_variant_parser`] macro:
//!
//! ```
//! # enum Op { Number(i64) }
//! #
//! #[derive(Clone, Debug, PartialEq, Eq, logos::Logos)]
//! enum Token {
//!     #[regex(r"-?[0-9]+", |lex| lex.slice().parse())]
//!     Number(i64),
//!
//!     // etc.
//! #   #[error]
//! #   Error,
//! }
//!
//! logos_nom_bridge::data_variant_parser! {
//!     fn parse_number(input) -> Result<Op>;
//!     pattern = Token::Number(n) => Op::Number(n);
//! }
//! ```

mod macros;

use core::fmt;

use logos::{Lexer, Logos, Span, SpannedIter};
use nom::{InputIter, InputLength, InputTake};

/// A [`logos::Lexer`] wrapper than can be used as an input for
/// [nom](https://docs.rs/nom/7.0.0/nom/index.html).
///
/// You can find an example in the [module-level docs](..).
pub struct Tokens<'i, T>
where
    T: Logos<'i>,
{
    lexer: Lexer<'i, T>,
}

impl<'i, T> Clone for Tokens<'i, T>
where
    T: Logos<'i> + Clone,
    T::Extras: Clone,
{
    fn clone(&self) -> Self {
        Self {
            lexer: self.lexer.clone(),
        }
    }
}

impl<'i, T> Tokens<'i, T>
where
    T: Logos<'i, Source = str> + Clone,
    T::Extras: Default + Clone,
{
    pub fn new(input: &'i str) -> Self {
        Tokens {
            lexer: Lexer::new(input),
        }
    }

    pub fn len(&self) -> usize {
        self.lexer.source().len() - self.lexer.span().end
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn peek(&self) -> Option<(T, &'i str)> {
        let mut iter = self.lexer.clone().spanned();
        iter.next().map(|(t, span)| (t, &self.lexer.source()[span]))
    }

    pub fn advance(mut self) -> Self {
        self.lexer.next();
        self
    }
}

impl<'i, T> PartialEq for Tokens<'i, T>
where
    T: PartialEq + Logos<'i> + Clone,
    T::Extras: Clone,
{
    fn eq(&self, other: &Self) -> bool {
        Iterator::eq(self.lexer.clone(), other.lexer.clone())
    }
}

impl<'i, T> Eq for Tokens<'i, T>
where
    T: Eq + Logos<'i> + Clone,
    T::Extras: Clone,
{
}

impl<'i, T> fmt::Debug for Tokens<'i, T>
where
    T: fmt::Debug + Logos<'i, Source = str>,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let source = self.lexer.source();
        let start = self.lexer.span().start;
        f.debug_tuple("Tokens").field(&&source[start..]).finish()
    }
}

impl<'i, T> Default for Tokens<'i, T>
where
    T: Logos<'i, Source = str>,
    T::Extras: Default,
{
    fn default() -> Self {
        Tokens {
            lexer: Lexer::new(""),
        }
    }
}

/// An iterator, that (similarly to [`std::iter::Enumerate`]) produces byte offsets of the tokens.
pub struct IndexIterator<'i, T>
where
    T: Logos<'i>,
{
    logos: Lexer<'i, T>,
}

impl<'i, T> Iterator for IndexIterator<'i, T>
where
    T: Logos<'i>,
{
    type Item = (usize, (T, Span));

    fn next(&mut self) -> Option<Self::Item> {
        self.logos.next().map(|t| {
            let span = self.logos.span();
            (span.start, (t, span))
        })
    }
}

impl<'i, T> InputIter for Tokens<'i, T>
where
    T: Logos<'i, Source = str> + Clone,
    T::Extras: Default + Clone,
{
    type Item = (T, Span);

    type Iter = IndexIterator<'i, T>;

    type IterElem = SpannedIter<'i, T>;

    fn iter_indices(&self) -> Self::Iter {
        IndexIterator {
            logos: self.lexer.clone(),
        }
    }

    fn iter_elements(&self) -> Self::IterElem {
        self.lexer.clone().spanned()
    }

    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        let mut iter = self.lexer.clone().spanned();
        iter.find(|t| predicate(t.clone()))
            .map(|(_, span)| span.start)
    }

    fn slice_index(&self, count: usize) -> Result<usize, nom::Needed> {
        let mut cnt = 0;
        for (_, span) in self.lexer.clone().spanned() {
            if cnt == count {
                return Ok(span.start);
            }
            cnt += 1;
        }
        if cnt == count {
            return Ok(self.len());
        }
        Err(nom::Needed::Unknown)
    }
}

impl<'i, T> InputLength for Tokens<'i, T>
where
    T: Logos<'i, Source = str> + Clone,
    T::Extras: Default + Clone,
{
    fn input_len(&self) -> usize {
        self.len()
    }
}

impl<'i, T> InputTake for Tokens<'i, T>
where
    T: Logos<'i, Source = str>,
    T::Extras: Default,
{
    fn take(&self, count: usize) -> Self {
        Tokens {
            lexer: Lexer::new(&self.lexer.source()[..count]),
        }
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        let (a, b) = self.lexer.source().split_at(count);
        (
            Tokens {
                lexer: Lexer::new(a),
            },
            Tokens {
                lexer: Lexer::new(b),
            },
        )
    }
}
