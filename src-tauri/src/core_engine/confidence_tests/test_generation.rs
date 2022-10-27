use crate::core_engine::syntax_tree::SwiftFunction;

pub fn generate_tests(function: SwiftFunction) {
    dbg!(function.get_parameters());
    dbg!(function.get_name());
}

#[cfg(test)]
mod tests {
    #[cfg(test)]
    mod generate_tests {
        use crate::core_engine::{
            features::bracket_highlight::utils::only_whitespace_on_line_until_position,
            utils::XcodeText, TextPosition,
        };

        #[test]
        fn out_of_range() {
            test_fn(
                "self.init(


                  forKnownProcessID: app.processIdentifier)",
                11,
                62,
                None,
            );
        }
    }
}
