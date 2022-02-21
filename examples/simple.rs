// First, create a `logos` lexer:

#[derive(Clone, Debug, PartialEq, Eq, logos::Logos)]
enum Token {
    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[regex(r"-?[0-9]+", |lex| lex.slice().parse())]
    Number(i64),

    #[error]
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Error,
}

// Then, write a nom parser that accepts a `Tokens<'_, Token>` as input:

use logos_nom_bridge::{data_variant_parser, token_parser, Tokens};

type Input<'source> = Tokens<'source, Token>;

#[derive(Debug, PartialEq, Eq)]
enum Op {
    Number(i64),
    Addition(Box<(Op, Op)>),
    Subtraction(Box<(Op, Op)>),
}

use nom::{
    branch::alt,
    combinator::{eof, map},
    sequence::tuple,
};

token_parser!(token: Token);

data_variant_parser! {
    fn parse_number(input) -> Result<Op>;
    pattern = Token::Number(n) => Op::Number(n);
}

fn parse_expression(input: Input<'_>) -> nom::IResult<Input<'_>, Op> {
    alt((
        map(
            tuple((
                parse_number,
                alt((Token::Plus, Token::Minus)),
                parse_expression,
                eof,
            )),
            |(a, op, b, _)| {
                if op == "+" {
                    Op::Addition(Box::new((a, b)))
                } else {
                    Op::Subtraction(Box::new((a, b)))
                }
            },
        ),
        parse_number,
    ))(input)
}

// Finally, you can use it to parse a string:

fn main() {
    let input = "10 + 3 - 4";
    let tokens = Tokens::new(input);

    let (rest, parsed) = parse_expression(tokens).unwrap();

    assert!(rest.is_empty());
    assert_eq!(
        parsed,
        Op::Addition(Box::new((
            Op::Number(10),
            Op::Subtraction(Box::new((Op::Number(3), Op::Number(4),))),
        ))),
    )
}
