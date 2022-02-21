/// Automatically implements [`nom::Parser`] for your token type.
///
/// ### Example
///
/// ```
/// #[derive(Clone, Debug, PartialEq, Eq, logos::Logos)]
/// enum Token {
///     #[token("test")]
///     Test,
///
///     #[error]
///     Error,
/// }
///
/// logos_nom_bridge::token_parser!(token: Token);
/// ```
///
/// You can use your own error type:
///
/// ```
/// # #[derive(Clone, Debug, PartialEq, Eq, logos::Logos)]
/// # enum Token {
/// #     #[token("test")]
/// #     Test,
/// #
/// #     #[error]
/// #     Error,
/// # }
/// #
/// logos_nom_bridge::token_parser!(
///     token: Token,
///     error: MyError = MyError::WrongToken,
/// );
///
/// enum MyError {
///     WrongToken,
/// }
/// ```
///
/// It's possible to store the input and/or the expected token in the error:
///
/// ```
/// # #[derive(Clone, Copy, Debug, PartialEq, Eq, logos::Logos)]
/// # enum Token {
/// #     #[token("test")]
/// #     Test,
/// #
/// #     #[error]
/// #     Error,
/// # }
/// #
/// logos_nom_bridge::token_parser!(
///     token: Token,
///     error<'src>(input, token): MyError<'src> = MyError::WrongToken {
///         input,
///         expected: *token,
///     }
/// );
///
/// enum MyError<'src> {
///     WrongToken {
///         input: logos_nom_bridge::Tokens<'src, Token>,
///         expected: Token,
///     },
/// }
/// ```
#[macro_export]
macro_rules! token_parser {
    (
        token: $token_ty:ty $(,)?
    ) => {
        $crate::token_parser!(
            token: $token_ty,
            error<'source>(input, token): ::nom::error::Error<$crate::Tokens<'source, $token_ty>> =
                nom::error::Error::new(input, nom::error::ErrorKind::IsA),
        );
    };

    (
        token: $token_ty:ty,
        error: $error_ty:ty = $error:expr $(,)?
    ) => {
        $crate::token_parser!(
            token: $token_ty,
            error<'source>(input, token): $error_ty = $error,
        );
    };

    (
        token: $token_ty:ty,
        error<$lt:lifetime>($input:ident, $token:ident): $error_ty:ty = $error:expr $(,)?
    ) => {
        impl<$lt> ::nom::Parser<
            $crate::Tokens<$lt, $token_ty>,
            &$lt str,
            $error_ty,
        > for $token_ty {
            fn parse(
                &mut self,
                $input: $crate::Tokens<$lt, $token_ty>,
            ) -> ::nom::IResult<
                $crate::Tokens<$lt, $token_ty>,
                &$lt str,
                $error_ty,
            > {
                match $input.peek() {
                    ::std::option::Option::Some((__token, __s)) if __token == *self => {
                        ::std::result::Result::Ok(($input.advance(), __s))
                    }
                    _ => {
                        let $token = self;
                        ::std::result::Result::Err(::nom::Err::Error($error))
                    },
                }
            }
        }
    };
}

/// Generates a nom parser function to parse an enum variant that contains data.
///
/// ### Example
///
/// ```
/// #[derive(Clone, Debug, PartialEq, Eq, logos::Logos)]
/// enum Token {
///     #[regex(r"-?[0-9]+", |lex| lex.slice().parse())]
///     Number(i64),
///
///     // etc.
/// #   #[error]
/// #   Error,
/// }
///
/// enum Op {
///     Number(i64),
///     // etc.
/// }
///
/// // Parse a `Token::Number` and return it as an `Op::Number`
/// logos_nom_bridge::data_variant_parser! {
///     fn parse_number(input) -> Result<Op>;
///     pattern = Token::Number(n) => Op::Number(n);
/// }
/// ```
///
/// And with a custom error:
///
/// ```
/// # #[derive(Clone, Debug, PartialEq, Eq, logos::Logos)]
/// # enum Token {
/// #     #[regex(r"-?[0-9]+", |lex| lex.slice().parse())]
/// #     Number(i64),
/// #
/// #     #[error]
/// #     Error,
/// # }
/// #
/// # enum Op {
/// #     Number(i64),
/// # }
/// #
/// enum MyError {
///     WrongToken,
///     // etc.
/// }
///
/// logos_nom_bridge::data_variant_parser! {
///     fn parse_number(input) -> Result<Op, MyError>;
///
///     pattern = Token::Number(n) => Op::Number(n);
///     error = MyError::WrongToken;
/// }
/// ```
///
/// It's possible to store the input in the error:
///
/// ```
/// # #[derive(Clone, Debug, PartialEq, Eq, logos::Logos)]
/// # enum Token {
/// #     #[regex(r"-?[0-9]+", |lex| lex.slice().parse())]
/// #     Number(i64),
/// #
/// #     #[error]
/// #     Error,
/// # }
/// #
/// # enum Op {
/// #     Number(i64),
/// # }
/// #
/// use logos_nom_bridge::Tokens;
///
/// enum MyError<'src> {
///     WrongToken {
///         input: Tokens<'src, Token>,
///     },
///     // etc.
/// }
///
/// logos_nom_bridge::data_variant_parser! {
///     fn parse_number<'src>(input) -> Result<Op, MyError<'src>>;
///
///     pattern = Token::Number(n) => Op::Number(n);
///     error = MyError::WrongToken { input };
/// }
/// ```
#[macro_export]
macro_rules! data_variant_parser {
    (
        fn $fn_name:ident($input:ident) -> Result<$ok_ty:ty>;

        pattern = $type:ident :: $variant:ident $data:tt => $res:expr;
    ) => {
        $crate::data_variant_parser! {
            fn $fn_name<'src>($input) -> Result<
                $ok_ty,
                ::nom::error::Error<$crate::Tokens<'src, $type>>,
            >;

            pattern = $type :: $variant $data => $res;
            error = ::nom::error::Error::new($input, ::nom::error::ErrorKind::IsA);
        }
    };

    (
        fn $fn_name:ident($input:ident) -> Result<$ok_ty:ty, $error_ty:ty $(,)?>;

        pattern = $type:ident :: $variant:ident $data:tt => $res:expr;
        error = $error:expr;
    ) => {
        $crate::data_variant_parser! {
            fn $fn_name<'src>($input) -> Result<$ok_ty, $error_ty>;

            pattern = $type :: $variant $data => $res;
            error = $error;
        }
    };

    (
        fn $fn_name:ident<$lt:lifetime>($input:ident) -> Result<$ok_ty:ty, $error_ty:ty $(,)?>;

        pattern = $type:ident :: $variant:ident $data:tt => $res:expr;
        error = $error:expr;
    ) => {
        fn $fn_name<$lt>($input: $crate::Tokens<$lt, $type>) -> ::nom::IResult<
            $crate::Tokens<$lt, $type>,
            $ok_ty,
            $error_ty,
        > {
            match $input.peek() {
                ::std::option::Option::Some(($type::$variant $data, _)) => {
                    Ok(($input.advance(), $res))
                }
                _ => ::std::result::Result::Err(::nom::Err::Error($error)),
            }
        }
    };
}
