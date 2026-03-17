use crate::rule::RuleExpr;

mod modules;
mod rule;

#[zyn::attribute()]
pub fn rule(
    #[zyn(input)] vis: zyn::Extract<zyn::syn::Visibility>,
    #[zyn(input)] ident: zyn::syn::ItemStruct,
    #[zyn(input)] attr: RuleExpr,
) -> zyn::TokenStream {
    let ident = ident.ident;
    let vis = vis.inner();
    zyn::zyn! {
        {{ vis }} struct {{ ident }} {
            inner: Box<dyn ::binary_options_tools_core::traits::Rule + Send + Sync>
        }

        impl {{ ident }} {
            {{ vis }} fn new() -> Self {
                Self {
                    inner: {{ attr.to_tokens() }},
                }
            }
        }


        impl ::binary_options_tools_core::traits::Rule for {{ ident }} {
            fn call(&self, msg: &::binary_options_tools_core::reimports::Message) -> bool {
                self.inner.call(msg)
            }

            fn reset(&self) {
                self.inner.reset()
            }
        }
    }
}
