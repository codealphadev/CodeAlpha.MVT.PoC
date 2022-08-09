use std::str::FromStr;

pub enum SwiftCodeBlockType {
    For,
    If,
    Else,
    Class,
    Function,
    Switch,
    While,
    Do,
    Guard,
}

impl FromStr for SwiftCodeBlockType {
    type Err = ();

    fn from_str(input: &str) -> Result<SwiftCodeBlockType, Self::Err> {
        match input {
            "for_statement" => Ok(SwiftCodeBlockType::For),
            "if_statement" => Ok(SwiftCodeBlockType::If),
            "else_statement" => Ok(SwiftCodeBlockType::Else),
            "class_body" => Ok(SwiftCodeBlockType::Class),
            "function_body" => Ok(SwiftCodeBlockType::Function),
            "switch_statement" => Ok(SwiftCodeBlockType::Switch),
            "while_statement" => Ok(SwiftCodeBlockType::While),
            "do_statement" => Ok(SwiftCodeBlockType::Do),
            "guard_statement" => Ok(SwiftCodeBlockType::Guard),
            _ => Err(()),
        }
    }
}
