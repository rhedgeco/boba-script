use crate::{
    parser::{node::NodeBuilderExt, report::PError, Node, TokenSource},
    Token,
};

use super::{Expr, Ident};

#[derive(Debug)]
pub struct LetVar {
    pub ident: Node<Ident>,
    pub expr: Node<Expr>,
}

impl LetVar {
    pub fn parse<'a>(source: &mut impl TokenSource<'a>) -> Result<Node<Self>, PError> {
        let mut builder = source.node_builder();

        // match let
        match builder.take() {
            Some((Token::Let, _)) => (),
            Some((token, span)) => {
                return Err(PError::UnexpectedToken {
                    expect: format!("'{}'", Token::Let),
                    found: format!("'{token}'"),
                    span: span.clone(),
                }
                .into())
            }
            None => {
                return Err(PError::UnexpectedEnd {
                    expect: format!("'{}'", Token::Let),
                    pos: builder.pos(),
                }
                .into())
            }
        }

        // parse ident
        let ident = Ident::parse(&mut builder)?;

        // match assign
        match builder.take() {
            Some((Token::Assign, _)) => (),
            Some((token, span)) => {
                return Err(PError::UnexpectedToken {
                    expect: format!("'{}'", Token::Assign),
                    found: format!("'{token}'"),
                    span: span.clone(),
                }
                .into())
            }
            None => {
                return Err(PError::UnexpectedEnd {
                    expect: format!("'{}'", Token::Assign),
                    pos: builder.pos(),
                }
                .into())
            }
        }

        // parse expression until end
        let expr = Expr::parse(&mut builder)?;

        // build letvar
        Ok(builder.build(Self { ident, expr }))
    }
}
