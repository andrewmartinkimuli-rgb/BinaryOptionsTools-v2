use zyn::{
    FromInput, Input, syn::{
        self, Expr, ExprClosure, ExprPath, Ident, LitStr, Token, Type, braced, parenthesized,
        parse::Parse, token::Paren,
    }
};

pub(crate) enum RuleExpr {
    /// A direct reference to a struct type: `rule(MyCustomRule)`
    Struct(Ident),

    /// A function path that returns a rule: `rule(fn = my_rule_maker)`
    FnRef(ExprPath),

    /// The actual DSL expression: `rule(starts_with("...") -> any())`
    Dsl(DslNode),
}

pub(crate) enum DslNode {
    /// `a -> b`
    Then(Box<DslNode>, Box<DslNode>),

    /// `a | b | c`
    Or(Vec<DslNode>),

    /// `a & b & c`
    And(Vec<DslNode>),

    /// `!a`
    Not(Box<DslNode>),

    /// The terminal node representing a matcher and its chained methods
    MethodChain(MethodChain),
}

pub(crate) struct MethodChain {
    pub base: MatcherCall,
    pub methods: Vec<ChainedMethod>,
}

pub(crate) enum MatcherCall {
    Any,                // any()
    Never,              // never()
    Exact(String),      // text_exact("string")
    StartsWith(String), // text_starts_with("string")
    EndsWith(String),   // text_ends_with("string")
    Contains(String),   // text_contains("string")
    Regex(String),      // text_regex("pattern")

    BinaryExact(Expr),      // binary_exact([0x01, 0x02])
    BinaryStartsWith(Expr), // binary_starts_with([0x01])
    BinaryEndsWith(Expr),   // binary_ends_with([0x01])
    BinaryContains(Expr),   // binary_contains([0x01])

    MessageType(Ident), // message_type(Text)
    JsonSchema(Type),   // json_schema(MyStruct)

    Custom(ExprClosure), // custom(|msg| { ... })
}

pub(crate) struct ChainedMethod {
    pub ident: Ident,    // e.g., `wait`, `lstrip_until`
    pub args: Vec<Expr>, // e.g., `[Expr(1)]`, `[Expr("{")]`
}

impl FromInput for RuleExpr {
    fn from_input(input: &Input) -> zyn::Result<Self> {
        // Pull the raw token stream from the attribute args and parse it
        let attr = input
            .attrs()
            .iter()
            .find(|a| a.path().is_ident("rule"))
            .ok_or_else(|| {
                zyn::syn::Error::new(zyn::Span::call_site(), "missing #[rule(...)] attribute")
            })?;

        Ok(attr.parse_args::<RuleExpr>()?)
    }
}

impl Parse for RuleExpr {
    fn parse(input: zyn::syn::parse::ParseStream) -> zyn::syn::Result<Self> {
        if input.peek(Ident) {
            Ok(Self::Struct(input.parse()?))
        } else if input.peek(Token![fn]) {
            let _: Token![fn] = input.parse()?;
            let _: Token![=] = input.parse()?;
            Ok(Self::FnRef(input.parse()?))
        } else if input.peek(zyn::syn::token::Brace) {
            let content;
            braced!(content in input);
            Ok(Self::Dsl(content.parse()?))
        } else {
            Err(zyn::syn::Error::new(
                input.span(),
                "Expected either a struct ident, a function reference (fn = ...), or a DSL expression!",
            ))
        }
    }
}

impl RuleExpr {
    pub fn to_tokens(&self) -> zyn::TokenStream {
        match self {
            Self::Struct(ident) => zyn::zyn! { ::std::boxed::Box::new({{ident}}::new()) },
            Self::FnRef(path) => zyn::zyn! { ::std::boxed::Box::new({{path}}()) },
            Self::Dsl(node) => zyn::zyn! { ::std::boxed::Box::new({{node.to_tokens()}}) },
        }
        .into()
    }
}

impl Parse for DslNode {
    fn parse(input: zyn::syn::parse::ParseStream) -> zyn::syn::Result<Self> {
        Self::parse_then(input)
    }
}

impl DslNode {
    fn parse_then(input: zyn::syn::parse::ParseStream) -> syn::Result<Self> {
        // 1. Go down one level to get the left side
        let mut node = Self::parse_or(input)?;

        // 2. While the NEXT token is `->`
        while input.peek(Token![->]) {
            input.parse::<Token![->]>()?; // consume `->`

            // 3. Go down one level to get the right side
            let right = Self::parse_or(input)?;

            // 4. Wrap them
            node = DslNode::Then(Box::new(node), Box::new(right));
        }

        Ok(node)
    }

    fn parse_or(input: zyn::syn::parse::ParseStream) -> syn::Result<Self> {
        // 1. Go down to AND
        let mut nodes = vec![Self::parse_and(input)?];

        // 2. Collect all `|` separated items into a flat Vec
        while input.peek(Token![|]) {
            input.parse::<Token![|]>()?;
            nodes.push(Self::parse_and(input)?);
        }

        if nodes.len() == 1 {
            Ok(nodes.pop().unwrap())
        } else {
            Ok(DslNode::Or(nodes))
        }
    }

    fn parse_and(input: zyn::syn::parse::ParseStream) -> syn::Result<Self> {
        // 1. Go down to NOT
        let mut nodes = vec![Self::parse_not(input)?];

        while input.peek(Token![&]) {
            input.parse::<Token![&]>()?;
            nodes.push(Self::parse_not(input)?);
        }

        if nodes.len() == 1 {
            Ok(nodes.pop().unwrap())
        } else {
            Ok(DslNode::And(nodes))
        }
    }

    fn parse_not(input: zyn::syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(Token![!]) {
            input.parse::<Token![!]>()?;
            // A NOT applies to the next Base element (which could be parenthesis!)
            let inner = Self::parse_base(input)?;
            Ok(DslNode::Not(Box::new(inner)))
        } else {
            // No `!`, just drop to the base level
            Self::parse_base(input)
        }
    }

    fn parse_base(input: zyn::syn::parse::ParseStream) -> syn::Result<Self> {
        // If it's a Parenthesis `(`
        if input.peek(syn::token::Paren) {
            let content;
            syn::parenthesized!(content in input);

            // CRITICAL: We restart the parsing cycle INSIDE the parenthesis
            // by calling the top-level parse function!
            let inner_node = DslNode::parse_then(&content)?;
            return Ok(inner_node);
        }

        // If it's not a parenthesis, it MUST be a MethodChain
        let chain = MethodChain::parse(input)?;
        Ok(DslNode::MethodChain(chain))
    }

    fn to_tokens(&self) -> zyn::TokenStream {
        match self {
            Self::And(nodes) => {
                if nodes.is_empty() {
                    return zyn::zyn! {panic!("AND operator requires at least one operand")}.into();
                }
                let first = &nodes[0];
                let rest = &nodes[1..];
                zyn::zyn! {
                    {{ first.to_tokens_inner() }}
                    @for (node in rest.iter()) {
                        .and({{ node.to_tokens_inner() }}.build())
                    }.build()
                }
            }
            Self::Or(nodes) => {
                if nodes.is_empty() {
                    return zyn::zyn! {panic!("OR operator requires at least one operand")}.into();
                }
                let first = &nodes[0];
                let rest = &nodes[1..];
                zyn::zyn! {
                    {{ first.to_tokens_inner() }}
                    @for (node in rest.iter()) {
                        .or({{ node.to_tokens_inner() }}.build())
                    }.build()
                }
            }
            Self::Then(node1, node2) => {
                zyn::zyn! {
                    {{ node1.to_tokens_inner() }}.then({{ node2.to_tokens_inner() }}.build()).build()
                }
            }
            Self::Not(node) => {
                zyn::zyn! {
                    {{ node.to_tokens_inner() }}.not().build()
                }
            }
            Self::MethodChain(node) => {
                zyn::zyn! {
                    {{ node.to_tokens() }}.build()
                }
            }
        }
        .into()
    }

    fn to_tokens_inner(&self) -> zyn::TokenStream {
        match self {
            Self::And(nodes) => {
                if nodes.is_empty() {
                    return zyn::zyn! {panic!("AND operator requires at least one operand")}.into();
                }
                let first = &nodes[0];
                let rest = &nodes[1..];
                zyn::zyn! {
                    {{ first.to_tokens_inner() }}
                    @for (node in rest.iter()) {
                        .and({{ node.to_tokens_inner() }}.build())
                    }
                }
                .into()
            }
            Self::Or(nodes) => {
                if nodes.is_empty() {
                    return zyn::zyn! {panic!("OR operator requires at least one operand")}.into();
                }
                let first = &nodes[0];
                let rest = &nodes[1..];
                zyn::zyn! {
                    {{ first.to_tokens_inner() }}
                    @for (node in rest.iter()) {
                        .or({{ node.to_tokens_inner() }}.build())
                    }
                }
                .into()
            }
            Self::Then(node1, node2) => zyn::zyn! {
                {{ node1.to_tokens_inner() }}.then({{ node2.to_tokens_inner() }}.build())
            }
            .into(),
            Self::Not(node) => zyn::zyn! {
                {{ node.to_tokens_inner() }}.not()
            }
            .into(),
            Self::MethodChain(node) => {
                // Return just the RuleBuilder without .build() - will be built by parent
                zyn::zyn! {
                    {{ node.to_tokens() }}
                }
                .into()
            }
        }
    }
}

impl Parse for MethodChain {
    fn parse(input: zyn::syn::parse::ParseStream) -> zyn::syn::Result<Self> {
        let base: MatcherCall = input.parse()?;
        let mut methods: Vec<ChainedMethod> = Vec::new();
        while input.peek(Token![.]) {
            let _: Token![.] = input.parse()?;
            methods.push(input.parse()?);
        }
        Ok(Self { base, methods })
    }
}

impl MethodChain {
    fn to_tokens(&self) -> zyn::TokenStream {
        zyn::zyn! {
            ::binary_options_tools_core::rules::RuleBuilder::{{ self.base.to_tokens() }}
            @for (method in self.methods.iter()) {
                .{{ method.to_tokens() }}
            }            
        }
        .into()
    }
}

impl Parse for MatcherCall {
    fn parse(input: zyn::syn::parse::ParseStream) -> zyn::syn::Result<Self> {
        if input.peek(Ident) {
            let ident: Ident = input.parse()?;
            if input.peek(Paren) {
                let content;
                let _ = parenthesized!(content in input);
                match ident.to_string().as_str() {
                    "any" => {
                        if content.is_empty() {
                            Ok(Self::Any)
                        } else {
                            Err(zyn::syn::Error::new(
                                ident.span(),
                                "Matcher 'any' does not take any arguments",
                            ))
                        }
                    }
                    "never" => {
                        if content.is_empty() {
                            Ok(Self::Never)
                        } else {
                            Err(zyn::syn::Error::new(
                                ident.span(),
                                "Matcher 'never' does not take any arguments",
                            ))
                        }
                    }
                    "exact" => Ok(Self::Exact(content.parse::<LitStr>()?.value())),
                    "starts_with" => Ok(Self::StartsWith(content.parse::<LitStr>()?.value())),
                    "ends_with" => Ok(Self::EndsWith(content.parse::<LitStr>()?.value())),
                    "contains" => Ok(Self::Contains(content.parse::<LitStr>()?.value())),
                    "regex" => Ok(Self::Regex(content.parse::<LitStr>()?.value())),

                    "binary_exact" => Ok(Self::BinaryExact(content.parse()?)),
                    "binary_starts_with" => Ok(Self::BinaryStartsWith(content.parse()?)),
                    "binary_ends_with" => Ok(Self::BinaryEndsWith(content.parse()?)),
                    "binary_contains" => Ok(Self::BinaryContains(content.parse()?)),

                    "message_type" => Ok(Self::MessageType(content.parse()?)),
                    "json_schema" => Ok(Self::JsonSchema(content.parse()?)),

                    "custom" => Ok(Self::Custom(content.parse()?)),

                    _ => Err(zyn::syn::Error::new(
                        ident.span(),
                        format!("Unknown matcher '{}'", ident),
                    )),
                }
            } else {
                Err(zyn::syn::Error::new(
                    ident.span(),
                    format!("Expected parenthesis after matcher '{}'", ident),
                ))
            }
        } else {
            Err(zyn::syn::Error::new(
                input.span(),
                "Expected an ident for matcher!",
            ))
        }
    }
}

impl MatcherCall {
    fn to_tokens(&self) -> zyn::TokenStream {
        match self {
            Self::Any => zyn::zyn! { any() }.into(),
            Self::Never => zyn::zyn! { never() }.into(),
            Self::Exact(s) => zyn::zyn! { text_exact({{s}}) }.into(),
            Self::StartsWith(s) => zyn::zyn! { text_starts_with({{s}}) }.into(),
            Self::EndsWith(s) => zyn::zyn! { text_ends_with({{s}}) }.into(),
            Self::Contains(s) => zyn::zyn! { text_contains({{s}}) }.into(),
            Self::Regex(s) => zyn::zyn! { text_regex({{s}}) }.into(),

            Self::BinaryExact(expr) => zyn::zyn! { binary_exact({{expr}}) }.into(),
            Self::BinaryStartsWith(expr) => zyn::zyn! { binary_starts_with({{expr}}) }.into(),
            Self::BinaryEndsWith(expr) => zyn::zyn! { binary_ends_with({{expr}}) }.into(),
            Self::BinaryContains(expr) => zyn::zyn! { binary_contains({{expr}}) }.into(),

            Self::MessageType(ident) => zyn::zyn! { message_type({{ident}}) }.into(),
            Self::JsonSchema(ty) => zyn::zyn! { json_schema::<{{ty}}>() }.into(),

            Self::Custom(closure) => zyn::zyn! { custom({{closure}}) }.into(),
        }
    }
}

impl Parse for ChainedMethod {
    fn parse(input: zyn::syn::parse::ParseStream) -> zyn::syn::Result<Self> {
        if input.peek(Ident) {
            let ident: Ident = input.parse()?;
            if input.peek(Paren) {
                let content;
                parenthesized!(content in input);
                let mut args: Vec<Expr> = Vec::new();

                // Parse comma-separated expressions from the parenthesis content
                if !content.is_empty() {
                    loop {
                        // Try to parse an expression
                        match content.parse::<Expr>() {
                            Ok(expr) => args.push(expr),
                            Err(_) => {
                                // If we can't parse an expression, break
                                // This handles edge cases gracefully
                                break;
                            }
                        }

                        // Check for comma
                        if content.peek(Token![,]) {
                            content.parse::<Token![,]>()?;
                        } else {
                            break;
                        }
                    }
                }
                return Ok(Self { ident, args });
            }
            return Err(zyn::syn::Error::new(
                input.span(),
                format!("Expected parenthesis after ident '{}'", ident),
            ));
        }
        Err(zyn::syn::Error::new(input.span(), "Expected ident!"))
    }
}

impl ChainedMethod {
    fn to_tokens(&self) -> zyn::TokenStream {
        let ident = &self.ident;
        let args = &self.args;
        zyn::zyn! {
            {{ ident }} (
                @for (arg in args) {
                    {{ arg }},
                }
            )
        }
        .into()
    }
}
