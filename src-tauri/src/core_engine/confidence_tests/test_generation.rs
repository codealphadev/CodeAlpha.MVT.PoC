use crate::core_engine::syntax_tree::SwiftFunction;

pub fn generate_tests(function: SwiftFunction) {
    dbg!(function.get_parameters());
    dbg!(function.get_name());
}

#[cfg(test)]
mod tests {
    #[cfg(test)]
    mod generate_tests {
        use crate::core_engine::{utils::XcodeText, TextPosition};

        #[test]
        fn out_of_range() {
            let code = XcodeText::from_str(
                r#"
                func fibonacciRecursiveNum1(num1: Int, num2: Int, steps: Int) {

                    if steps > 0 {
                        let newNum = num1 + num2
                        fibonacciRecursiveNum1(num2, num2: newNum, steps: steps-1)
                    }
                    else {
                        print("result = \(num2)")
                    }
                }
            "#,
            );
        }
    }
}
