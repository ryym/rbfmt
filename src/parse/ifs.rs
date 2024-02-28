use crate::fmt;

use super::postmodifiers;

struct IfOrUnless<'src> {
    is_if: bool,
    loc: prism::Location<'src>,
    predicate: prism::Node<'src>,
    statements: Option<prism::StatementsNode<'src>>,
    consequent: Option<prism::Node<'src>>,
    end_loc: Option<prism::Location<'src>>,
}

impl<'src> super::Parser<'src> {
    pub(super) fn parse_if_or_ternary(&mut self, node: prism::IfNode) -> fmt::Node {
        if node.end_keyword_loc().is_some() {
            self.parse_if_or_unless(IfOrUnless {
                is_if: true,
                loc: node.location(),
                predicate: node.predicate(),
                statements: node.statements(),
                consequent: node.consequent(),
                end_loc: node.end_keyword_loc(),
            })
        } else if node.then_keyword_loc().map(|l| l.as_slice()) == Some(b"?") {
            let ternary = self.visit_ternary(node);
            fmt::Node::new(fmt::Kind::Ternary(ternary))
        } else {
            self.parse_postmodifier(postmodifiers::Postmodifier {
                keyword: "if".to_string(),
                keyword_loc: node.if_keyword_loc().expect("if modifier must have if"),
                predicate: node.predicate(),
                statements: node.statements(),
            })
        }
    }

    pub(super) fn parse_unless(&mut self, node: prism::UnlessNode) -> fmt::Node {
        if node.end_keyword_loc().is_some() {
            self.parse_if_or_unless(IfOrUnless {
                is_if: false,
                loc: node.location(),
                predicate: node.predicate(),
                statements: node.statements(),
                consequent: node.consequent().map(|n| n.as_node()),
                end_loc: node.end_keyword_loc(),
            })
        } else {
            self.parse_postmodifier(postmodifiers::Postmodifier {
                keyword: "unless".to_string(),
                keyword_loc: node.keyword_loc(),
                predicate: node.predicate(),
                statements: node.statements(),
            })
        }
    }

    fn parse_if_or_unless(&mut self, node: IfOrUnless) -> fmt::Node {
        let if_leading = self.take_leading_trivia(node.loc.start_offset());

        let end_loc = node.end_loc.expect("if/unless expression must have end");
        let end_start = end_loc.start_offset();

        let conseq = node.consequent;
        let next_pred_loc_start = node
            .statements
            .as_ref()
            .map(|s| s.location())
            .or_else(|| conseq.as_ref().map(|c| c.location()))
            .map(|l| l.start_offset())
            .unwrap_or(end_loc.start_offset());
        let predicate = self.parse(node.predicate, Some(next_pred_loc_start));

        let ifexpr = match conseq {
            // if...(elsif...|else...)+end
            Some(conseq) => {
                // take trailing of else/elsif
                let else_start = conseq.location().start_offset();
                let body = self.parse_statements_body(node.statements, Some(else_start));
                let if_first = fmt::Conditional::new(predicate, body);
                let mut ifexpr = fmt::If::new(node.is_if, if_first);
                self.parse_ifelse(conseq, &mut ifexpr);
                ifexpr
            }
            // if...end
            None => {
                let body = self.parse_statements_body(node.statements, Some(end_start));
                let if_first = fmt::Conditional::new(predicate, body);
                fmt::If::new(node.is_if, if_first)
            }
        };

        let mut node = fmt::Node::new(fmt::Kind::If(ifexpr));
        node.prepend_leading_trivia(if_leading);
        node
    }

    fn parse_ifelse(&mut self, node: prism::Node, ifexpr: &mut fmt::If) {
        match node {
            // elsif ("if" only, "unles...elsif" is syntax error)
            prism::Node::IfNode { .. } => {
                let node = node.as_if_node().unwrap();

                let end_loc = node
                    .end_keyword_loc()
                    .expect("if/unless expression must have end");

                let predicate = node.predicate();
                let consequent = node.consequent();

                let predicate_next = node
                    .statements()
                    .map(|s| s.location().start_offset())
                    .or_else(|| consequent.as_ref().map(|c| c.location().start_offset()))
                    .unwrap_or(end_loc.start_offset());
                let predicate = self.parse(predicate, Some(predicate_next));

                let body_end_loc = consequent
                    .as_ref()
                    .map(|n| n.location().start_offset())
                    .unwrap_or(end_loc.start_offset());
                let body = self.parse_statements_body(node.statements(), Some(body_end_loc));

                let conditional = fmt::Conditional::new(predicate, body);
                ifexpr.elsifs.push(conditional);
                if let Some(consequent) = consequent {
                    self.parse_ifelse(consequent, ifexpr);
                }
            }
            // else
            prism::Node::ElseNode { .. } => {
                let node = node.as_else_node().unwrap();
                let end_loc = node
                    .end_keyword_loc()
                    .expect("if/unless expression must have end");
                let if_last = self.parse_else(node, end_loc.start_offset());
                ifexpr.if_last = Some(if_last);
            }
            _ => {
                panic!("unexpected node in IfNode: {:?}", node);
            }
        }
    }

    fn visit_ternary(&mut self, node: prism::IfNode) -> fmt::Ternary {
        let question_loc = node.then_keyword_loc().expect("ternary if must have ?");
        let predicate = self.parse(node.predicate(), Some(question_loc.start_offset()));
        let then = node
            .statements()
            .and_then(|s| s.body().iter().next())
            .expect("ternary if must have then statement");
        match node.consequent() {
            Some(consequent) => match consequent {
                prism::Node::ElseNode { .. } => {
                    let consequent = consequent.as_else_node().unwrap();
                    let otherwise = consequent
                        .statements()
                        .and_then(|s| s.body().iter().next())
                        .expect("ternary if must have else statement");
                    let pred_trailing = self.take_trailing_comment(then.location().start_offset());
                    let loc = consequent.location();
                    let then = self.parse(then, Some(loc.start_offset()));
                    let otherwise = self.parse(otherwise, None);
                    fmt::Ternary::new(predicate, pred_trailing, then, otherwise)
                }
                _ => panic!("ternary if consequent must be ElseNode: {:?}", node),
            },
            _ => panic!("ternary if must have consequent"),
        }
    }
}
