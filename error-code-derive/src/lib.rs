mod error_info;

use error_info::process_error_info;
use proc_macro::TokenStream;

// ToErrorInfo 就是我们取的过程名字，attributes 里面是我们允许的参数，会在语法树中捕获
// 函数需要输入一个 TokenStream 流语法树，我们用 quote 重组语法树然后再输出
// 具体在 process_error_info 中进行流的控制，加我们的功能
#[proc_macro_derive(ToErrorInfo, attributes(error_info))]
pub fn derive_to_error_info(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    process_error_info(input).into()
}
