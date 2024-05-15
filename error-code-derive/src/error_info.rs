use darling::{
    ast::{Data, Fields, Style},
    util, FromDeriveInput, FromVariant,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

// 首先要创建一个结构体，用来存储我们的参数
// 注意要加上 attribute error_info，这样我们的参数才能被捕获
// ident, generics, data 是固定的，用来存储我们的 enum 名字，泛型，数据
// app_type 是我们的 app_code 类型，prefix 是我们的错误码前缀，是自己在使用时候定义的
#[derive(Debug, FromDeriveInput)]
#[darling(attributes(error_info))]
struct ErrorData {
    ident: syn::Ident,
    generics: syn::Generics,
    data: Data<EnumVariants, ()>,
    app_type: syn::Type,
    prefix: String,
}

// 然后创建一个结构体，用来存储我们的 enum 的每一个 variant 的参数
// 同样 ident 和 fields 是固定的，用来存储我们的 variant 名字和字段
// 一开始这里我们不关心 fields 里的字段，所以用 Ignored，不过下面还是用 fields.style 来判断
// code, app_code, client_msg 是我们的参数，注意这里要加上 default，因为我们的参数是可选的，如果没有就用默认值
#[derive(Debug, FromVariant)]
#[darling(attributes(error_info))]
struct EnumVariants {
    ident: syn::Ident,
    fields: Fields<util::Ignored>,
    code: String,
    #[darling(default)]
    app_code: String,
    #[darling(default)]
    client_msg: String,
}

// 这里基本上边打印边调试，随时用 cargo expand 和 cargo test 测试
pub(crate) fn process_error_info(input: DeriveInput) -> TokenStream {
    let ErrorData {
        ident: name,
        generics,
        data: Data::Enum(data),
        app_type,
        prefix,
    } = ErrorData::from_derive_input(&input).expect("Can not parse input")
    else {
        panic!("Only enum is supported");
    };

    // for each variant, generate a match arm
    // #name::#ident(_) => { // code to new ErrorInfo }
    let code = data
        .iter()
        .map(|v| {
            let EnumVariants {
                ident,
                fields,
                code,
                app_code,
                client_msg,
            } = v;
            let code = format!("{}{}", prefix, code);
            let varint_code = match fields.style {
                Style::Struct => quote! { #name::#ident { .. } },
                Style::Tuple => quote! { #name::#ident(_) },
                Style::Unit => quote! { #name::#ident },
            };

            quote! {
                #varint_code => {
                    ErrorInfo::new(
                        #app_code,
                        #code,
                        #client_msg,
                        self,
                    )
                }
            }
        })
        .collect::<Vec<_>>();

    quote! {
        use error_code::{ErrorInfo, ToErrorInfo as _};
        impl #generics ToErrorInfo for #name #generics {
            type T = #app_type;

            fn to_error_info(&self) -> ErrorInfo<Self::T> {
                match self {
                    #(#code),*
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_data_struct() {
        let input = r#"
        #[derive(thiserror::Error, ToErrorInfo)]
        #[error_info(app_type="http::StatusCode", prefix="01")]
        pub enum MyError {
        #[error("Invalid command: {0}")]
        #[error_info(code="IC", app_code="400")]
        InvalidCommand(String),

        #[error("Invalid argument: {0}")]
        #[error_info(code="IA", app_code="400", client_msg="friendly msg")]
        InvalidArgument(String),

        #[error("{0}")]
        #[error_info(code="RE", app_code="500")]
        RespError(#[from] RespError),
        }
        "#;

        let parsed = syn::parse_str(input).unwrap();
        let info = ErrorData::from_derive_input(&parsed).unwrap();
        println!("{:#?}", info);

        assert_eq!(info.ident.to_string(), "MyError");
        assert_eq!(info.prefix, "01");

        let code = process_error_info(parsed);
        println!("{}", code);
    }
}
