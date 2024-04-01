use std::borrow::Cow;
use std::collections::HashMap;

use itertools::Itertools;
use pretty::BoxDoc;
use typst_syntax::{ast, SyntaxNode};
use typst_syntax::{ast::*, SyntaxKind};

use crate::attr::Attributes;
use crate::util::{pretty_items, FoldStyle};

#[derive(Debug, Default)]
pub struct PrettyPrinter {
    attr_map: HashMap<SyntaxNode, Attributes>,
}

impl PrettyPrinter {
    pub fn new(attr_map: HashMap<SyntaxNode, Attributes>) -> Self {
        Self { attr_map }
    }
}

impl PrettyPrinter {
    pub fn convert_markup<'a>(&'a self, root: Markup<'a>) -> BoxDoc<'a, ()> {
        let mut doc: BoxDoc<()> = BoxDoc::nil();
        #[derive(Debug, Default)]
        struct Line<'a> {
            has_text: bool,
            nodes: Vec<&'a SyntaxNode>,
        }
        // break markup into lines, split by stmt, parbreak, newline, multiline raw,
        // equation if a line contains text, it will be skipped by the formatter
        // to keep the original format
        let lines = {
            let mut lines: Vec<Line> = vec![];
            let mut current_line = Line {
                has_text: false,
                nodes: vec![],
            };
            for node in root.to_untyped().children() {
                let mut break_line = false;
                if let Some(space) = node.cast::<Space>() {
                    if space.to_untyped().text().contains('\n') {
                        break_line = true;
                    }
                } else if let Some(pb) = node.cast::<Parbreak>() {
                    if pb.to_untyped().text().contains('\n') {
                        break_line = true;
                    }
                } else if node.kind().is_stmt() {
                    break_line = true;
                } else if let Some(expr) = node.cast::<Expr>() {
                    match expr {
                        ast::Expr::Text(_) => current_line.has_text = true,
                        ast::Expr::Raw(r) => {
                            if r.block() {
                                break_line = true;
                            } else {
                                current_line.has_text = true;
                            }
                        }
                        ast::Expr::Strong(_) | ast::Expr::Emph(_) => current_line.has_text = true,
                        ast::Expr::Code(_) => break_line = true,
                        ast::Expr::Equation(e) if e.block() => break_line = true,
                        _ => (),
                    }
                }
                current_line.nodes.push(node);
                if break_line {
                    lines.push(current_line);
                    current_line = Line::default();
                }
            }
            if !current_line.nodes.is_empty() {
                lines.push(current_line);
            }
            lines
        };
        for Line { has_text, nodes } in lines {
            for node in nodes {
                if let Some(space) = node.cast::<Space>() {
                    doc = doc.append(self.convert_space(space));
                    continue;
                }
                if let Some(pb) = node.cast::<Parbreak>() {
                    doc = doc.append(self.convert_parbreak(pb));
                    continue;
                }
                if has_text {
                    doc = doc.append(self.format_disabled(node));
                } else if let Some(expr) = node.cast::<Expr>() {
                    let expr_doc = self.convert_expr(expr);
                    doc = doc.append(expr_doc);
                } else {
                    doc = doc.append(trivia(node));
                }
            }
        }
        doc
    }

    fn check_disabled<'a>(&'a self, node: &'a SyntaxNode) -> Option<BoxDoc<'a, ()>> {
        match self.attr_map.get(node) {
            Some(attr) if attr.no_format() => Some(self.format_disabled(node)),
            _ => None,
        }
    }

    fn format_disabled<'a>(&'a self, node: &'a SyntaxNode) -> BoxDoc<'a, ()> {
        return BoxDoc::text(node.clone().into_text().to_string());
    }

    fn convert_expr<'a>(&'a self, expr: Expr<'a>) -> BoxDoc<'a, ()> {
        if let Some(res) = self.check_disabled(expr.to_untyped()) {
            return res;
        }
        match expr {
            ast::Expr::Text(t) => self.convert_text(t),
            ast::Expr::Space(s) => self.convert_space(s),
            ast::Expr::Linebreak(b) => self.convert_linebreak(b),
            ast::Expr::Parbreak(b) => self.convert_parbreak(b),
            ast::Expr::Escape(e) => self.convert_escape(e),
            ast::Expr::Shorthand(s) => self.convert_shorthand(s),
            ast::Expr::SmartQuote(s) => self.convert_smart_quote(s),
            ast::Expr::Strong(s) => self.convert_strong(s),
            ast::Expr::Emph(e) => self.convert_emph(e),
            ast::Expr::Raw(r) => self.convert_raw(r),
            ast::Expr::Link(l) => self.convert_link(l),
            ast::Expr::Label(l) => self.convert_label(l),
            ast::Expr::Ref(r) => self.convert_ref(r),
            ast::Expr::Heading(h) => self.convert_heading(h),
            ast::Expr::List(l) => self.convert_list_item(l),
            ast::Expr::Enum(e) => self.convert_enum_item(e),
            ast::Expr::Term(t) => self.convert_term_item(t),
            ast::Expr::Equation(e) => self.convert_equation(e),
            ast::Expr::Math(m) => self.convert_math(m),
            ast::Expr::MathIdent(mi) => trivia(mi.to_untyped()),
            ast::Expr::MathAlignPoint(map) => trivia(map.to_untyped()),
            ast::Expr::MathDelimited(md) => self.convert_math_delimited(md),
            ast::Expr::MathAttach(ma) => self.convert_math_attach(ma),
            ast::Expr::MathPrimes(mp) => self.convert_math_primes(mp),
            ast::Expr::MathFrac(mf) => self.convert_math_frac(mf),
            ast::Expr::MathRoot(mr) => self.convert_math_root(mr),
            ast::Expr::Ident(i) => self.convert_ident(i),
            ast::Expr::None(n) => self.convert_none(n),
            ast::Expr::Auto(a) => self.convert_auto(a),
            ast::Expr::Bool(b) => self.convert_bool(b),
            ast::Expr::Int(i) => self.convert_int(i),
            ast::Expr::Float(f) => self.convert_float(f),
            ast::Expr::Numeric(n) => self.convert_numeric(n),
            ast::Expr::Str(s) => self.convert_str(s),
            ast::Expr::Code(c) => self.convert_code_block(c),
            ast::Expr::Content(c) => self.convert_content_block(c),
            ast::Expr::Parenthesized(p) => self.convert_parenthesized(p),
            ast::Expr::Array(a) => self.convert_array(a),
            ast::Expr::Dict(d) => self.convert_dict(d),
            ast::Expr::Unary(u) => self.convert_unary(u),
            ast::Expr::Binary(b) => self.convert_binary(b),
            ast::Expr::FieldAccess(fa) => self.convert_field_access(fa),
            ast::Expr::FuncCall(fc) => self.convert_func_call(fc),
            ast::Expr::Closure(c) => self.convert_closure(c),
            ast::Expr::Let(l) => self.convert_let_binding(l),
            ast::Expr::DestructAssign(da) => self.convert_destruct_assignment(da),
            ast::Expr::Set(s) => self.convert_set_rule(s),
            ast::Expr::Show(s) => self.convert_show_rule(s),
            ast::Expr::Conditional(c) => self.convert_conditional(c),
            ast::Expr::While(w) => self.convert_while(w),
            ast::Expr::For(f) => self.convert_for(f),
            ast::Expr::Import(i) => self.convert_import(i),
            ast::Expr::Include(i) => self.convert_include(i),
            ast::Expr::Break(b) => self.convert_break(b),
            ast::Expr::Continue(c) => self.convert_continue(c),
            ast::Expr::Return(r) => self.convert_return(r),
            ast::Expr::Contextual(c) => self.convert_contextual(c),
        }
        .group()
    }

    fn convert_text<'a>(&'a self, text: Text<'a>) -> BoxDoc<'a, ()> {
        let node = text.to_untyped();
        trivia(node)
    }

    fn convert_space<'a>(&'a self, space: Space<'a>) -> BoxDoc<'a, ()> {
        let node = space.to_untyped();
        if node.text().contains('\n') {
            BoxDoc::hardline()
        } else {
            BoxDoc::space()
        }
    }

    fn convert_linebreak<'a>(&'a self, linebreak: Linebreak<'a>) -> BoxDoc<'a, ()> {
        let node = linebreak.to_untyped();
        trivia(node)
    }

    fn convert_parbreak<'a>(&'a self, _parbreak: Parbreak<'a>) -> BoxDoc<'a, ()> {
        BoxDoc::hardline().append(BoxDoc::hardline())
    }

    fn convert_escape<'a>(&'a self, escape: Escape<'a>) -> BoxDoc<'a, ()> {
        let node = escape.to_untyped();
        trivia(node)
    }

    fn convert_shorthand<'a>(&'a self, shorthand: Shorthand<'a>) -> BoxDoc<'a, ()> {
        let node = shorthand.to_untyped();
        trivia(node)
    }

    fn convert_smart_quote<'a>(&'a self, smart_quote: SmartQuote<'a>) -> BoxDoc<'a, ()> {
        let node = smart_quote.to_untyped();
        trivia(node)
    }

    fn convert_strong<'a>(&'a self, strong: Strong<'a>) -> BoxDoc<'a, ()> {
        let body = self.convert_markup(strong.body());
        BoxDoc::text("*").append(body).append(BoxDoc::text("*"))
    }

    fn convert_emph<'a>(&'a self, emph: Emph<'a>) -> BoxDoc<'a, ()> {
        let body = self.convert_markup(emph.body());
        BoxDoc::text("_").append(body).append(BoxDoc::text("_"))
    }

    fn convert_raw<'a>(&'a self, raw: Raw<'a>) -> BoxDoc<'a, ()> {
        let mut doc = BoxDoc::nil();
        let is_block = raw.block();
        let has_lang = raw.lang().is_some();
        let mut is_opening = true;
        let mut is_first_text = true;
        let mut last_text: Option<Text> = None;
        for child in raw.to_untyped().children() {
            if let Some(delim) = child.cast::<RawDelim>() {
                // deal with single line raw that ends with `
                if !is_block && !is_opening {
                    if let Some(last_text) = last_text {
                        if last_text.get().ends_with('`') {
                            doc = doc.append(BoxDoc::space());
                        }
                    }
                }
                doc = doc.append(trivia(delim.to_untyped()));
                if is_block && !has_lang && is_opening {
                    doc = doc.append(BoxDoc::hardline());
                }
                is_opening = false;
            }
            if let Some(lang) = child.cast::<RawLang>() {
                doc = doc.append(trivia(lang.to_untyped()));
                doc = doc.append(if is_block {
                    BoxDoc::hardline()
                } else {
                    BoxDoc::space()
                });
            }
            if let Some(line) = child.cast::<Text>() {
                // deal with single line raw that starts with `
                if is_first_text && line.get().starts_with('`') && !is_block && !has_lang {
                    doc = doc.append(BoxDoc::space());
                }
                is_first_text = false;
                last_text = Some(line);
                doc = doc.append(trivia(line.to_untyped()));
                if is_block {
                    doc = doc.append(BoxDoc::hardline());
                }
            }
        }
        doc
    }

    fn convert_link<'a>(&'a self, link: Link<'a>) -> BoxDoc<'a, ()> {
        let node = link.to_untyped();
        trivia(node)
    }

    fn convert_label<'a>(&'a self, label: Label<'a>) -> BoxDoc<'a, ()> {
        let node = label.to_untyped();
        trivia(node)
    }

    fn convert_ref<'a>(&'a self, reference: Ref<'a>) -> BoxDoc<'a, ()> {
        let mut doc = BoxDoc::text("@");
        doc = doc.append(BoxDoc::text(reference.target()));
        if let Some(supplement) = reference.supplement() {
            doc = doc.append(self.convert_content_block(supplement));
        }
        doc
    }

    fn convert_heading<'a>(&'a self, heading: Heading<'a>) -> BoxDoc<'a, ()> {
        let mut doc = BoxDoc::text("=".repeat(heading.depth().into()));
        doc = doc.append(BoxDoc::space());
        doc = doc.append(self.convert_markup(heading.body()));
        doc
    }

    fn convert_list_item<'a>(&'a self, list_item: ListItem<'a>) -> BoxDoc<'a, ()> {
        let mut doc = BoxDoc::text("-");
        doc = doc.append(BoxDoc::space());
        doc = doc.append(self.convert_markup(list_item.body()).nest(2));
        doc
    }

    fn convert_enum_item<'a>(&'a self, enum_item: EnumItem<'a>) -> BoxDoc<'a, ()> {
        let mut doc = if let Some(number) = enum_item.number() {
            BoxDoc::text(format!("{number}."))
        } else {
            BoxDoc::text("+")
        };
        doc = doc.append(BoxDoc::space());
        doc = doc.append(self.convert_markup(enum_item.body()).nest(2));
        doc
    }

    fn convert_term_item<'a>(&'a self, term: TermItem<'a>) -> BoxDoc<'a, ()> {
        let mut doc = BoxDoc::text("/");
        doc = doc.append(BoxDoc::space());
        doc = doc.append(self.convert_markup(term.term()));
        doc = doc.append(BoxDoc::text(":"));
        doc = doc.append(BoxDoc::space());
        doc = doc.append(self.convert_markup(term.description()).nest(2));
        doc
    }

    fn convert_equation<'a>(&'a self, equation: Equation<'a>) -> BoxDoc<'a, ()> {
        let mut doc = BoxDoc::text("$");
        if equation.block() {
            doc = doc.append(BoxDoc::line());
        }
        doc = doc.append(self.convert_math(equation.body()).nest(2));
        if equation.block() {
            doc = doc.append(BoxDoc::line());
        }
        doc = doc.append(BoxDoc::text("$"));
        doc
    }

    fn convert_math<'a>(&'a self, math: Math<'a>) -> BoxDoc<'a, ()> {
        if let Some(res) = self.check_disabled(math.to_untyped()) {
            return res;
        }
        let mut doc: BoxDoc<()> = BoxDoc::nil();
        for node in math.to_untyped().children() {
            if let Some(expr) = node.cast::<Expr>() {
                let expr_doc = self.convert_expr(expr);
                doc = doc.append(expr_doc);
            } else if let Some(space) = node.cast::<Space>() {
                doc = doc.append(self.convert_space(space));
            } else {
                doc = doc.append(trivia(node));
            }
        }
        doc
    }

    fn convert_ident<'a>(&'a self, ident: Ident<'a>) -> BoxDoc<'a, ()> {
        let doc = BoxDoc::nil().append(BoxDoc::text(ident.as_str()));
        doc
    }

    fn convert_none<'a>(&'a self, _none: None<'a>) -> BoxDoc<'a, ()> {
        BoxDoc::nil().append(BoxDoc::text("none"))
    }

    fn convert_auto<'a>(&'a self, _auto: Auto<'a>) -> BoxDoc<'a, ()> {
        BoxDoc::nil().append(BoxDoc::text("auto"))
    }

    fn convert_bool<'a>(&'a self, boolean: Bool<'a>) -> BoxDoc<'a, ()> {
        let node = boolean.to_untyped();
        trivia(node)
    }

    fn convert_int<'a>(&'a self, int: Int<'a>) -> BoxDoc<'a, ()> {
        let node = int.to_untyped();
        trivia(node)
    }

    fn convert_float<'a>(&'a self, float: Float<'a>) -> BoxDoc<'a, ()> {
        let node = float.to_untyped();
        trivia(node)
    }

    fn convert_numeric<'a>(&'a self, numeric: Numeric<'a>) -> BoxDoc<'a, ()> {
        let node = numeric.to_untyped();
        trivia(node)
    }

    fn convert_str<'a>(&'a self, str: Str<'a>) -> BoxDoc<'a, ()> {
        let node = str.to_untyped();
        trivia(node)
    }

    fn convert_code_block<'a>(&'a self, code_block: CodeBlock<'a>) -> BoxDoc<'a, ()> {
        let mut codes: Vec<_> = vec![];
        for node in code_block.to_untyped().children() {
            if let Some(code) = node.cast::<Code>() {
                let code_doc = self.convert_code(code);
                codes.extend(code_doc);
            } else if node.kind() == SyntaxKind::LineComment
                || node.kind() == SyntaxKind::BlockComment
            {
                codes.push(to_doc(std::borrow::Cow::Borrowed(node.text()), true));
            }
        }
        let doc = pretty_items(
            &codes,
            BoxDoc::text(";").append(BoxDoc::space()),
            BoxDoc::nil(),
            (BoxDoc::text("{"), BoxDoc::text("}")),
            true,
            FoldStyle::Never,
        );
        doc
    }

    fn convert_code<'a>(&'a self, code: Code<'a>) -> Vec<BoxDoc<'a, ()>> {
        let mut codes: Vec<_> = vec![];
        for node in code.to_untyped().children() {
            if let Some(expr) = node.cast::<Expr>() {
                let expr_doc = self.convert_expr(expr);
                codes.push(expr_doc);
            } else if node.kind() == SyntaxKind::LineComment
                || node.kind() == SyntaxKind::BlockComment
            {
                codes.push(to_doc(std::borrow::Cow::Borrowed(node.text()), true));
            } else if node.kind() == SyntaxKind::Space {
                let newline_cnt = node.text().chars().filter(|c| *c == '\n').count();
                for _ in 0..newline_cnt.saturating_sub(1) {
                    codes.push(BoxDoc::nil());
                }
            }
        }
        codes
    }

    fn convert_content_block<'a>(&'a self, content_block: ContentBlock<'a>) -> BoxDoc<'a, ()> {
        let content = self.convert_markup(content_block.body()).group().nest(2);
        let doc = BoxDoc::text("[").append(content).append(BoxDoc::text("]"));
        doc
    }

    fn convert_parenthesized<'a>(&'a self, parenthesized: Parenthesized<'a>) -> BoxDoc<'a, ()> {
        let mut doc = BoxDoc::text("(");
        let inner = self.convert_expr(parenthesized.expr());
        let multiline_expr = BoxDoc::line()
            .append(inner.clone())
            .append(BoxDoc::line())
            .nest(2)
            .group();
        let singleline_expr = inner;
        doc = doc.append(multiline_expr.flat_alt(singleline_expr));
        doc = doc.append(BoxDoc::text(")"));
        doc
    }

    fn convert_array<'a>(&'a self, array: Array<'a>) -> BoxDoc<'a, ()> {
        let array_items = array
            .items()
            .map(|item| self.convert_array_item(item))
            .collect_vec();
        if array_items.len() == 1 {
            let singleline = BoxDoc::text("(")
                .append(array_items[0].clone())
                .append(BoxDoc::text(","))
                .append(BoxDoc::text(")"));
            let multiline = BoxDoc::text("(")
                .append(
                    BoxDoc::hardline()
                        .append(array_items[0].clone())
                        .append(BoxDoc::text(","))
                        .nest(2),
                )
                .append(BoxDoc::hardline())
                .append(BoxDoc::text(")"))
                .group();
            multiline.flat_alt(singleline)
        } else {
            let style = FoldStyle::from_attr(self.attr_map.get(array.to_untyped()));

            pretty_items(
                &array_items,
                BoxDoc::text(",").append(BoxDoc::space()),
                BoxDoc::text(","),
                (BoxDoc::text("("), BoxDoc::text(")")),
                false,
                style,
            )
        }
    }

    fn convert_array_item<'a>(&'a self, array_item: ArrayItem<'a>) -> BoxDoc<'a, ()> {
        let doc = match array_item {
            ArrayItem::Pos(p) => self.convert_expr(p),
            ArrayItem::Spread(s) => self.convert_spread(s),
        };
        doc
    }

    fn convert_dict<'a>(&'a self, dict: Dict<'a>) -> BoxDoc<'a, ()> {
        if dict.items().count() == 0 {
            return BoxDoc::text("(:)");
        }
        let dict_items = dict
            .items()
            .map(|item| self.convert_dict_item(item))
            .collect_vec();
        let style = FoldStyle::from_attr(self.attr_map.get(dict.to_untyped()));
        pretty_items(
            &dict_items,
            BoxDoc::text(",").append(BoxDoc::space()),
            BoxDoc::text(","),
            (BoxDoc::text("("), BoxDoc::text(")")),
            false,
            style,
        )
    }

    fn convert_dict_item<'a>(&'a self, dict_item: DictItem<'a>) -> BoxDoc<'a, ()> {
        match dict_item {
            DictItem::Named(n) => self.convert_named(n),
            DictItem::Keyed(k) => self.convert_keyed(k),
            DictItem::Spread(s) => self.convert_spread(s),
        }
    }

    fn convert_named<'a>(&'a self, named: Named<'a>) -> BoxDoc<'a, ()> {
        if let Some(res) = self.check_disabled(named.to_untyped()) {
            return res;
        }
        // TODO: better handling hash #
        let has_hash = named
            .to_untyped()
            .children()
            .any(|node| matches!(node.kind(), SyntaxKind::Hash));
        let mut doc = self.convert_ident(named.name());
        doc = doc.append(BoxDoc::text(":"));
        doc = doc.append(BoxDoc::space());
        if has_hash {
            doc = doc.append(BoxDoc::text("#"));
        }
        doc = doc.append(self.convert_expr(named.expr()));
        doc
    }

    fn convert_keyed<'a>(&'a self, keyed: Keyed<'a>) -> BoxDoc<'a, ()> {
        let mut doc = self.convert_expr(keyed.key());
        doc = doc.append(BoxDoc::text(":"));
        doc = doc.append(BoxDoc::space());
        doc = doc.append(self.convert_expr(keyed.expr()));
        doc
    }

    fn convert_unary<'a>(&'a self, unary: Unary<'a>) -> BoxDoc<'a, ()> {
        let op_text = match unary.op() {
            UnOp::Pos => "+",
            UnOp::Neg => "-",
            UnOp::Not => "not ",
        };
        BoxDoc::text(op_text).append(self.convert_expr(unary.expr()))
    }

    fn convert_binary<'a>(&'a self, binary: Binary<'a>) -> BoxDoc<'a, ()> {
        BoxDoc::nil()
            .append(self.convert_expr(binary.lhs()))
            .append(BoxDoc::space())
            .append(BoxDoc::text(binary.op().as_str()))
            .append(BoxDoc::space())
            .append(self.convert_expr(binary.rhs()))
    }

    fn convert_field_access<'a>(&'a self, field_access: FieldAccess<'a>) -> BoxDoc<'a, ()> {
        let left = BoxDoc::nil().append(self.convert_expr(field_access.target()));
        let singleline_right = BoxDoc::text(".").append(self.convert_ident(field_access.field()));
        let _multiline_right = BoxDoc::hardline()
            .append(BoxDoc::text("."))
            .append(self.convert_ident(field_access.field()))
            .nest(2)
            .group();
        // TODO: typst doesn't support this
        // left.append(multiline_right.flat_alt(singleline_right))
        left.append(singleline_right)
    }

    fn convert_func_call<'a>(&'a self, func_call: FuncCall<'a>) -> BoxDoc<'a, ()> {
        let mut doc = BoxDoc::nil().append(self.convert_expr(func_call.callee()));
        if let Some(res) = self.check_disabled(func_call.args().to_untyped()) {
            return doc.append(res);
        }
        let has_parenthesized_args = func_call
            .args()
            .to_untyped()
            .children()
            .any(|node| matches!(node.kind(), SyntaxKind::LeftParen | SyntaxKind::RightParen));
        if has_parenthesized_args {
            let (args, prefer_tighter, is_multiline) =
                self.convert_parenthesized_args(func_call.args());

            doc = if prefer_tighter {
                doc.append(BoxDoc::text("("))
                    .append(args.into_iter().next().unwrap_or_else(BoxDoc::nil))
                    .append(BoxDoc::text(")"))
            } else {
                doc.append(pretty_items(
                    &args,
                    BoxDoc::text(",").append(BoxDoc::space()),
                    BoxDoc::text(","),
                    (BoxDoc::text("("), BoxDoc::text(")")),
                    false,
                    if is_multiline {
                        FoldStyle::Never
                    } else {
                        FoldStyle::Fit
                    },
                ))
            }
        };
        doc.append(self.convert_additional_args(func_call.args(), has_parenthesized_args))
    }

    fn convert_parenthesized_args<'a>(
        &'a self,
        args: Args<'a>,
    ) -> (Vec<BoxDoc<'a, ()>>, bool, bool) {
        let node = args.to_untyped();
        let mut last_arg = None;
        let mut is_multiline = false;
        for node in node
            .children()
            .take_while(|node| node.kind() != SyntaxKind::RightParen)
        {
            if let Some(space) = node.cast::<Space>() {
                is_multiline = is_multiline || space.to_untyped().text().contains('\n');
            }
        }
        let args: Vec<BoxDoc<'a, ()>> = node
            .children()
            .take_while(|node| node.kind() != SyntaxKind::RightParen)
            .filter_map(|node| node.cast::<'_, Arg>())
            .map(|arg| {
                last_arg = Some(arg);
                is_multiline = is_multiline
                    || self
                        .attr_map
                        .get(arg.to_untyped())
                        .map_or(false, |attr| attr.is_multiline_flavor());
                self.convert_arg(arg)
            })
            .collect();
        // We prefer tighter style if...
        // 1. There are no arguments
        // 2. There is only one argument and it is not a function call
        let prefer_tighter = args.is_empty()
            || (args.len() == 1 && {
                let arg = last_arg.unwrap();
                let rhs = match arg {
                    Arg::Pos(p) => p,
                    Arg::Named(n) => n.expr(),
                    Arg::Spread(s) => s.expr(),
                };
                !matches!(rhs, Expr::FuncCall(..))
            });
        (args, prefer_tighter, is_multiline)
    }

    fn convert_additional_args<'a>(&'a self, args: Args<'a>, has_paren: bool) -> BoxDoc<'a, ()> {
        let node = args.to_untyped();
        let args = node
            .children()
            .skip_while(|node| {
                if has_paren {
                    node.kind() != SyntaxKind::RightParen
                } else {
                    node.kind() != SyntaxKind::ContentBlock
                }
            })
            .filter_map(|node| node.cast::<'_, Arg>());
        BoxDoc::concat(args.map(|arg| self.convert_arg(arg))).group()
    }

    fn convert_arg<'a>(&'a self, arg: Arg<'a>) -> BoxDoc<'a, ()> {
        match arg {
            Arg::Pos(p) => self.convert_expr(p),
            Arg::Named(n) => self.convert_named(n),
            Arg::Spread(s) => self.convert_spread(s),
        }
    }

    fn convert_closure<'a>(&'a self, closure: Closure<'a>) -> BoxDoc<'a, ()> {
        let mut doc = BoxDoc::nil();
        let params = self.convert_params(closure.params());
        let style = FoldStyle::from_attr(self.attr_map.get(closure.params().to_untyped()));
        let arg_list = pretty_items(
            &params,
            BoxDoc::text(",").append(BoxDoc::space()),
            BoxDoc::text(","),
            (BoxDoc::text("("), BoxDoc::text(")")),
            false,
            style,
        );
        if let Some(name) = closure.name() {
            doc = doc.append(self.convert_ident(name));
            doc = doc.append(arg_list);
            doc = doc.append(BoxDoc::space());
            doc = doc.append(BoxDoc::text("="));
            doc = doc.append(BoxDoc::space());
            doc = doc.append(self.convert_expr(closure.body()));
        } else {
            if params.len() == 1
                && matches!(closure.params().children().next().unwrap(), Param::Pos(_))
                && !matches!(
                    closure.params().children().next().unwrap(),
                    Param::Pos(Pattern::Destructuring(_))
                )
            {
                doc = params[0].clone();
            } else {
                doc = arg_list
            }
            doc = doc.append(BoxDoc::space());
            doc = doc.append(BoxDoc::text("=>"));
            doc = doc.append(BoxDoc::space());
            doc = doc.append(self.convert_expr(closure.body()));
        }
        doc
    }

    fn convert_params<'a>(&'a self, params: Params<'a>) -> Vec<BoxDoc<'a, ()>> {
        params
            .children()
            .map(|param| self.convert_param(param))
            .collect()
    }

    fn convert_param<'a>(&'a self, param: Param<'a>) -> BoxDoc<'a, ()> {
        match param {
            Param::Pos(p) => self.convert_pattern(p),
            Param::Named(n) => self.convert_named(n),
            Param::Spread(s) => self.convert_spread(s),
        }
    }

    fn convert_spread<'a>(&'a self, spread: Spread<'a>) -> BoxDoc<'a, ()> {
        let mut doc = BoxDoc::text("..");
        let ident = if let Some(id) = spread.sink_ident() {
            self.convert_ident(id)
        } else if let Some(expr) = spread.sink_expr() {
            self.convert_expr(expr)
        } else {
            BoxDoc::nil()
        };
        doc = doc.append(ident);
        doc
    }

    fn convert_pattern<'a>(&'a self, pattern: Pattern<'a>) -> BoxDoc<'a, ()> {
        match pattern {
            Pattern::Normal(n) => self.convert_expr(n),
            Pattern::Placeholder(p) => self.convert_underscore(p),
            Pattern::Destructuring(d) => self.convert_destructuring(d),
            Pattern::Parenthesized(p) => self.convert_parenthesized(p),
        }
    }

    fn convert_underscore<'a>(&'a self, _underscore: Underscore<'a>) -> BoxDoc<'a, ()> {
        BoxDoc::text("_")
    }

    fn convert_destructuring<'a>(&'a self, destructuring: Destructuring<'a>) -> BoxDoc<'a, ()> {
        let items: Vec<_> = destructuring
            .items()
            .map(|item| self.convert_destructuring_item(item))
            .collect();
        if items.len() == 1
            && matches!(
                destructuring.items().next().unwrap(),
                DestructuringItem::Pattern(_)
            )
        {
            BoxDoc::text("(")
                .append(items.into_iter().next().unwrap())
                .append(BoxDoc::text(",)"))
        } else {
            pretty_items(
                &items,
                BoxDoc::text(",").append(BoxDoc::space()),
                BoxDoc::text(","),
                (BoxDoc::text("("), BoxDoc::text(")")),
                false,
                FoldStyle::Fit,
            )
        }
    }

    fn convert_destructuring_item<'a>(
        &'a self,
        destructuring_item: DestructuringItem<'a>,
    ) -> BoxDoc<'a, ()> {
        match destructuring_item {
            DestructuringItem::Spread(s) => self.convert_spread(s),
            DestructuringItem::Named(n) => self.convert_named(n),
            DestructuringItem::Pattern(p) => self.convert_pattern(p),
        }
    }

    fn convert_let_binding<'a>(&'a self, let_binding: LetBinding<'a>) -> BoxDoc<'a, ()> {
        let mut doc = BoxDoc::nil()
            .append(BoxDoc::text("let"))
            .append(BoxDoc::space());
        match let_binding.kind() {
            LetBindingKind::Normal(n) => {
                doc = doc.append(self.convert_pattern(n).group());
                if let Some(expr) = let_binding.init() {
                    doc = doc.append(BoxDoc::space());
                    doc = doc.append(BoxDoc::text("="));
                    doc = doc.append(BoxDoc::space());
                    doc = doc.append(self.convert_expr(expr));
                }
            }
            LetBindingKind::Closure(_c) => {
                if let Some(c) = let_binding.init() {
                    doc = doc.append(self.convert_expr(c));
                }
            }
        }
        doc
    }

    fn convert_destruct_assignment<'a>(
        &'a self,
        destruct_assign: DestructAssignment<'a>,
    ) -> BoxDoc<'a, ()> {
        self.convert_pattern(destruct_assign.pattern())
            .append(BoxDoc::space())
            .append(BoxDoc::text("="))
            .append(BoxDoc::space())
            .append(self.convert_expr(destruct_assign.value()))
    }

    fn convert_set_rule<'a>(&'a self, set_rule: SetRule<'a>) -> BoxDoc<'a, ()> {
        let mut doc = BoxDoc::nil()
            .append(BoxDoc::text("set"))
            .append(BoxDoc::space());
        doc = doc.append(self.convert_expr(set_rule.target()));
        let (args, _, _) = self.convert_parenthesized_args(set_rule.args());
        doc = doc.append(pretty_items(
            &args,
            BoxDoc::text(",").append(BoxDoc::space()),
            BoxDoc::text(","),
            (BoxDoc::text("("), BoxDoc::text(")")),
            false,
            FoldStyle::Single,
        ));
        if let Some(condition) = set_rule.condition() {
            doc = doc.append(BoxDoc::space());
            doc = doc.append(BoxDoc::text("if"));
            doc = doc.append(BoxDoc::space());
            doc = doc.append(self.convert_expr(condition));
        }
        doc
    }

    fn convert_show_rule<'a>(&'a self, show_rule: ShowRule<'a>) -> BoxDoc<'a, ()> {
        let mut doc = BoxDoc::nil().append(BoxDoc::text("show"));
        if let Some(selector) = show_rule.selector() {
            doc = doc.append(BoxDoc::space());
            doc = doc.append(self.convert_expr(selector));
        }
        doc = doc.append(BoxDoc::text(":"));
        doc = doc.append(BoxDoc::space());
        doc = doc.append(self.convert_expr(show_rule.transform()));
        doc
    }

    fn convert_conditional<'a>(&'a self, conditional: Conditional<'a>) -> BoxDoc<'a, ()> {
        let mut doc = BoxDoc::nil();
        enum CastType {
            Condition,
            Then,
            Else,
        }
        let has_else = conditional.else_body().is_some();
        let mut expr_type = CastType::Condition;
        for child in conditional.to_untyped().children() {
            if child.kind() == SyntaxKind::If {
                doc = doc.append(BoxDoc::text("if"));
                doc = doc.append(BoxDoc::space());
            } else if child.kind() == SyntaxKind::Else {
                doc = doc.append(BoxDoc::text("else"));
                doc = doc.append(BoxDoc::space());
            } else if child.kind() == SyntaxKind::BlockComment {
                doc = doc.append(trivia(child));
                doc = doc.append(BoxDoc::space());
            } else if child.kind() == SyntaxKind::LineComment {
                doc = doc.append(trivia(child));
                doc = doc.append(BoxDoc::hardline());
            } else {
                match expr_type {
                    CastType::Condition => {
                        if let Some(condition) = child.cast() {
                            doc = doc.append(self.convert_expr(condition));
                            doc = doc.append(BoxDoc::space());
                            expr_type = CastType::Then;
                        }
                    }
                    CastType::Then => {
                        if let Some(then_expr) = child.cast() {
                            doc = doc.append(self.convert_expr(then_expr).group());
                            if has_else {
                                expr_type = CastType::Else;
                                doc = doc.append(BoxDoc::space());
                            }
                        }
                    }
                    CastType::Else => {
                        if let Some(else_expr) = child.cast() {
                            doc = doc.append(self.convert_expr(else_expr).group());
                        }
                    }
                }
            }
        }
        doc
    }

    fn convert_while<'a>(&'a self, while_loop: WhileLoop<'a>) -> BoxDoc<'a, ()> {
        let mut doc = BoxDoc::nil();
        #[derive(Debug, PartialEq)]
        enum CastType {
            Condition,
            Body,
        }
        let mut expr_type = CastType::Condition;
        for child in while_loop.to_untyped().children() {
            if child.kind() == SyntaxKind::While {
                doc = doc.append(BoxDoc::text("while"));
                doc = doc.append(BoxDoc::space());
            } else if child.kind() == SyntaxKind::BlockComment {
                doc = doc.append(trivia(child));
                doc = doc.append(BoxDoc::space());
            } else if child.kind() == SyntaxKind::LineComment {
                doc = doc.append(trivia(child));
                doc = doc.append(BoxDoc::hardline());
            } else if let Some(expr) = child.cast() {
                doc = doc.append(self.convert_expr(expr));
                if expr_type == CastType::Condition {
                    doc = doc.append(BoxDoc::space());
                    expr_type = CastType::Body;
                }
            }
        }
        doc
    }

    fn convert_for<'a>(&'a self, for_loop: ForLoop<'a>) -> BoxDoc<'a, ()> {
        let mut doc = BoxDoc::nil();
        enum CastType {
            Pattern,
            Iter,
            Body,
        }
        let mut expr_type = CastType::Pattern;
        for child in for_loop.to_untyped().children() {
            if child.kind() == SyntaxKind::For {
                doc = doc.append(BoxDoc::text("for"));
                doc = doc.append(BoxDoc::space());
            } else if child.kind() == SyntaxKind::In {
                doc = doc.append(BoxDoc::text("in"));
                doc = doc.append(BoxDoc::space());
            } else if child.kind() == SyntaxKind::BlockComment {
                doc = doc.append(trivia(child));
                doc = doc.append(BoxDoc::space());
            } else if child.kind() == SyntaxKind::LineComment {
                doc = doc.append(trivia(child));
                doc = doc.append(BoxDoc::hardline());
            } else {
                match expr_type {
                    CastType::Pattern => {
                        if let Some(pattern) = child.cast() {
                            doc = doc.append(self.convert_pattern(pattern));
                            doc = doc.append(BoxDoc::space());
                            expr_type = CastType::Iter;
                        }
                    }
                    CastType::Iter => {
                        if let Some(iter) = child.cast() {
                            doc = doc.append(self.convert_expr(iter));
                            doc = doc.append(BoxDoc::space());
                            expr_type = CastType::Body;
                        }
                    }
                    CastType::Body => {
                        if let Some(body) = child.cast() {
                            doc = doc.append(self.convert_expr(body));
                        }
                    }
                }
            }
        }
        doc
    }

    fn convert_import<'a>(&'a self, import: ModuleImport<'a>) -> BoxDoc<'a, ()> {
        let mut doc = BoxDoc::nil().append(BoxDoc::text("import"));
        doc = doc.append(BoxDoc::space());
        doc = doc.append(self.convert_expr(import.source()));
        if let Some(new_name) = import.new_name() {
            doc = doc.append(BoxDoc::space());
            doc = doc.append(BoxDoc::text("as"));
            doc = doc.append(BoxDoc::space());
            doc = doc.append(self.convert_ident(new_name));
        }
        if let Some(imports) = import.imports() {
            doc = doc.append(BoxDoc::text(":"));
            doc = doc.append(BoxDoc::space());
            let imports = match imports {
                Imports::Wildcard => BoxDoc::text("*"),
                Imports::Items(i) => BoxDoc::intersperse(
                    i.iter().map(|item| self.convert_import_item(item)),
                    BoxDoc::text(",").append(BoxDoc::space()),
                ),
            };
            doc = doc.append(imports.group());
        }
        doc
    }

    fn convert_import_item<'a>(&'a self, import_item: ImportItem<'a>) -> BoxDoc<'a, ()> {
        match import_item {
            ImportItem::Simple(s) => self.convert_ident(s),
            ImportItem::Renamed(r) => self
                .convert_ident(r.original_name())
                .append(BoxDoc::space())
                .append(BoxDoc::text("as"))
                .append(BoxDoc::space())
                .append(self.convert_ident(r.new_name())),
        }
    }

    fn convert_include<'a>(&'a self, include: ModuleInclude<'a>) -> BoxDoc<'a, ()> {
        BoxDoc::nil()
            .append(BoxDoc::text("include"))
            .append(BoxDoc::space())
            .append(self.convert_expr(include.source()))
    }

    fn convert_break<'a>(&'a self, _break: LoopBreak<'a>) -> BoxDoc<'a, ()> {
        BoxDoc::nil().append(BoxDoc::text("break"))
    }

    fn convert_continue<'a>(&'a self, _continue: LoopContinue<'a>) -> BoxDoc<'a, ()> {
        BoxDoc::nil().append(BoxDoc::text("continue"))
    }

    fn convert_return<'a>(&'a self, return_stmt: FuncReturn<'a>) -> BoxDoc<'a, ()> {
        let mut doc = BoxDoc::nil()
            .append(BoxDoc::text("return"))
            .append(BoxDoc::space());
        if let Some(body) = return_stmt.body() {
            doc = doc.append(self.convert_expr(body));
        }
        doc
    }

    fn convert_math_delimited<'a>(&'a self, math_delimited: MathDelimited<'a>) -> BoxDoc<'a, ()> {
        let open = self.convert_expr(math_delimited.open());
        let close = self.convert_expr(math_delimited.close());
        let body = self.convert_math(math_delimited.body());
        let singleline = open.clone().append(body.clone()).append(close.clone());
        let multiline = open
            .append(BoxDoc::hardline())
            .append(body)
            .append(BoxDoc::hardline())
            .nest(2)
            .append(close);
        multiline.flat_alt(singleline)
    }

    fn convert_math_attach<'a>(&'a self, math_attach: MathAttach<'a>) -> BoxDoc<'a, ()> {
        let mut doc = self.convert_expr(math_attach.base());
        let prime_index = math_attach
            .to_untyped()
            .children()
            .enumerate()
            .skip_while(|(_i, node)| node.cast::<Expr<'_>>().is_none())
            .nth(1)
            .filter(|(_i, n)| n.cast::<MathPrimes>().is_some())
            .map(|(i, _n)| i);

        let bottom_index = math_attach
            .to_untyped()
            .children()
            .enumerate()
            .skip_while(|(_i, node)| !matches!(node.kind(), SyntaxKind::Underscore))
            .find_map(|(i, n)| SyntaxNode::cast::<Expr<'_>>(n).map(|n| (i, n)))
            .map(|(i, _n)| i);

        let top_index = math_attach
            .to_untyped()
            .children()
            .enumerate()
            .skip_while(|(_i, node)| !matches!(node.kind(), SyntaxKind::Hat))
            .find_map(|(i, n)| SyntaxNode::cast::<Expr<'_>>(n).map(|n| (i, n)))
            .map(|(i, _n)| i);

        #[derive(Debug)]
        enum IndexType {
            Prime,
            Bottom,
            Top,
        }

        let mut index_types = [IndexType::Prime, IndexType::Bottom, IndexType::Top];
        index_types.sort_by_key(|index_type| match index_type {
            IndexType::Prime => prime_index,
            IndexType::Bottom => bottom_index,
            IndexType::Top => top_index,
        });

        for index in index_types {
            match index {
                IndexType::Prime => {
                    if let Some(primes) = math_attach.primes() {
                        doc = doc.append(self.convert_math_primes(primes));
                    }
                }
                IndexType::Bottom => {
                    if let Some(bottom) = math_attach.bottom() {
                        doc = doc.append(BoxDoc::text("_"));
                        doc = doc.append(self.convert_expr(bottom));
                    }
                }
                IndexType::Top => {
                    if let Some(top) = math_attach.top() {
                        doc = doc.append(BoxDoc::text("^"));
                        doc = doc.append(self.convert_expr(top));
                    }
                }
            }
        }
        doc
    }

    fn convert_math_primes<'a>(&'a self, math_primes: MathPrimes<'a>) -> BoxDoc<'a, ()> {
        BoxDoc::text("'".repeat(math_primes.count()))
    }

    fn convert_math_frac<'a>(&'a self, math_frac: MathFrac<'a>) -> BoxDoc<'a, ()> {
        let singleline = self
            .convert_expr(math_frac.num())
            .append(BoxDoc::space())
            .append(BoxDoc::text("/"))
            .append(BoxDoc::space())
            .append(self.convert_expr(math_frac.denom()));
        // TODO: add multiline version
        singleline
    }

    fn convert_math_root<'a>(&'a self, math_root: MathRoot<'a>) -> BoxDoc<'a, ()> {
        let sqrt_sym = if let Some(index) = math_root.index() {
            if index == 3 {
                BoxDoc::text("∛")
            } else if index == 4 {
                BoxDoc::text("∜")
            } else {
                // TODO: actually unreachable
                BoxDoc::text("√")
            }
        } else {
            BoxDoc::text("√")
        };
        sqrt_sym.append(self.convert_expr(math_root.radicand()))
    }

    fn convert_contextual<'a>(&'a self, ctx: Contextual<'a>) -> BoxDoc<'a, ()> {
        let body = self.convert_expr(ctx.body());
        BoxDoc::text("context").append(BoxDoc::space()).append(body)
    }
}

fn trivia(node: &SyntaxNode) -> BoxDoc<'_, ()> {
    to_doc(std::borrow::Cow::Borrowed(node.text()), false)
}

pub fn to_doc(s: Cow<'_, str>, strip_prefix: bool) -> BoxDoc<'_, ()> {
    let get_line = |s: &str| {
        if strip_prefix {
            s.trim_start().to_string()
        } else {
            s.to_string()
        }
    };
    // String::lines() doesn't include the trailing newline
    let has_trailing_newline = s.ends_with('\n');
    let res = BoxDoc::intersperse(
        s.lines().map(|s| BoxDoc::text(get_line(s))),
        BoxDoc::hardline(),
    );
    if has_trailing_newline {
        res.append(BoxDoc::hardline())
    } else {
        res
    }
}

#[cfg(test)]
mod tests {
    use typst_syntax::parse;

    use super::*;

    #[test]
    fn test_to_doc() {
        let tests = [
            "command can take a directory as an argument to use as the book",
            "123\n456\n789",
            "123\n4567\n789\n",
            "123\n4568\n789\n",
        ];
        for test in tests.into_iter() {
            insta::assert_debug_snapshot!(to_doc(test.into(), false));
        }
    }

    #[test]
    fn convert_markup() {
        let tests = [r"=== --open

When you use the `--open` flag, typst-book will open the rendered book in
your default web browser after building it."];
        for test in tests.into_iter() {
            let root = parse(test);
            insta::assert_debug_snapshot!(root);
            let markup = root.cast().unwrap();
            let printer = PrettyPrinter::default();
            let doc = printer.convert_markup(markup);
            insta::assert_debug_snapshot!(doc.pretty(120).to_string());
        }
    }

    #[test]
    fn convert_func_call() {
        let tests = [r#"#link("http://example.com")[test]"#];
        for test in tests.into_iter() {
            let root = parse(test);
            insta::assert_debug_snapshot!(root);
            let markup = root.cast().unwrap();
            let printer = PrettyPrinter::default();
            let doc = printer.convert_markup(markup);
            insta::assert_debug_snapshot!(doc.pretty(120).to_string());
        }
    }
}
