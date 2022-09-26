use crate::core_engine::XcodeText;

// TODO: Refactor together with FunctionParameter
#[derive(Clone, Debug)]
pub struct SwiftFunctionParameter {
    //internal_name: XcodeText,
    pub external_name: XcodeText,
    pub var_type: XcodeText,
    //in_out: bool, // TODO
}
#[derive(Clone, Debug)]
pub enum SwiftFunctionContext {
    FilePrivate,
}

#[derive(Clone, Debug)]
pub struct SwiftFunctionComponents {
    pub body: XcodeText,
    pub parameters: Vec<SwiftFunctionParameter>,
    pub name: XcodeText,
    pub return_type: Option<XcodeText>,
    pub context: SwiftFunctionContext,
}

pub fn reconstruct_function(components: SwiftFunctionComponents) -> XcodeText {
    let mut result = XcodeText::new_empty();

    match components.context {
        SwiftFunctionContext::FilePrivate => result += "fileprivate ",
    }

    result += XcodeText::from_str("func ") + components.name + "(";

    for (i, parameter) in components.parameters.iter().enumerate() {
        result += XcodeText::from_str("_ ")
            + parameter.external_name.clone()
            + ": "
            + parameter.var_type.clone();

        if i != components.parameters.len() - 1 {
            result += ", ";
        }
    }
    result += ")";
    // TODO: Async, throws
    if let Some(return_type) = components.return_type {
        result += XcodeText::from_str(" -> ") + return_type;
    }
    result += " {\n";

    result += components.body;
    result += "\n}\n\n";

    result
}

#[cfg(test)]
mod tests {
    mod reconstruct_function {
        use crate::core_engine::{
            syntax_tree::{
                reconstruct_function, swift::reconstruction::function::SwiftFunctionContext,
                SwiftFunctionComponents, SwiftFunctionParameter,
            },
            XcodeText,
        };

        #[test]
        fn simple_case() {
            let body = r#"    var num1 = 0
    var num2 = 1
    
    for _ in 0 ..< n {
        let num = num1 + num2
        num1 = num2
        num2 = num
    }
    print("result = \(num2)")"#;

            let expected = r#"fileprivate func fib(_ n: Int32) {
    var num1 = 0
    var num2 = 1
    
    for _ in 0 ..< n {
        let num = num1 + num2
        num1 = num2
        num2 = num
    }
    print("result = \(num2)")
}

"#;
            let components = SwiftFunctionComponents {
                body: XcodeText::from_str(body),
                parameters: vec![SwiftFunctionParameter {
                    external_name: XcodeText::from_str("n"),
                    var_type: XcodeText::from_str("Int32"),
                }],
                name: XcodeText::from_str("fib"),
                return_type: None,
                context: SwiftFunctionContext::FilePrivate,
            };
            assert_eq!(
                reconstruct_function(components),
                XcodeText::from_str(expected)
            );
        }
        #[test]
        fn case_with_output() {
            let body = r#"    var num1 = n2
    var num2 = 1
    
    for _ in 0 ..< n {
        let num = num1 + num2
        num1 = num2
        num2 = num
    }
    return num2;"#;

            let expected = r#"fileprivate func fib(_ n: Int32, _ n2: String) -> Int32 {
    var num1 = n2
    var num2 = 1
    
    for _ in 0 ..< n {
        let num = num1 + num2
        num1 = num2
        num2 = num
    }
    return num2;
}

"#;
            let components = SwiftFunctionComponents {
                body: XcodeText::from_str(body),
                parameters: vec![
                    SwiftFunctionParameter {
                        external_name: XcodeText::from_str("n"),
                        var_type: XcodeText::from_str("Int32"),
                    },
                    SwiftFunctionParameter {
                        external_name: XcodeText::from_str("n2"),
                        var_type: XcodeText::from_str("String"),
                    },
                ],
                name: XcodeText::from_str("fib"),
                return_type: Some(XcodeText::from_str("Int32")),
                context: SwiftFunctionContext::FilePrivate,
            };
            assert_eq!(
                reconstruct_function(components),
                XcodeText::from_str(expected)
            );
        }
    }
}
