use std::collections::HashMap;

use tree_sitter::Node;

use crate::core_engine::{
    syntax_tree::{calculate_cognitive_complexities, get_node_text, Complexities, SwiftSyntaxTree},
    XcodeText,
};

use super::ComplexityRefactoringError;

#[derive(Clone, Debug)]
struct Scope<'a> {
    variables: Vec<XcodeText>,
    node: Node<'a>,
}

#[derive(Clone, Debug)]
struct PossibleMethodExtraction<'a> {
    nodes: Vec<Node<'a>>,
    outside_references: Vec<String>,
}

pub fn check_for_method_extraction<'a>(
    node: Node<'a>,
    text_content: &'a XcodeText,
    syntax_tree: &'a SwiftSyntaxTree,
) -> Result<Option<Vec<Node<'a>>>, ComplexityRefactoringError> {
    // Build up a list of possible nodes to extract, each with relevant metrics used for comparison

    let mut scopes: Vec<Scope> = Vec::new();
    scopes.push(Scope {
        variables: Vec::new(),
        node,
    });

    let possible_extractions: Vec<PossibleMethodExtraction> =
        evaluate_node(node, text_content, syntax_tree, &mut scopes)?;

    let function_complexity = syntax_tree
        .get_node_metadata(&node)
        .map_err(|e| ComplexityRefactoringError::GenericError(e.into()))?
        .complexities
        .clone();

    let (best_extraction, score) = get_best_extraction(
        possible_extractions,
        syntax_tree,
        text_content,
        function_complexity,
    )?;

    const SCORE_THRESHOLD: f64 = 1.0;

    if score > SCORE_THRESHOLD {
        return Ok(best_extraction.map(|e| e.nodes));
    } else {
        return Ok(None);
    }
}

fn get_best_extraction<'a>(
    possibilities: Vec<PossibleMethodExtraction<'a>>,
    syntax_tree: &'a SwiftSyntaxTree,
    text_content: &'a XcodeText,
    original_complexity: Complexities,
) -> Result<(Option<PossibleMethodExtraction<'a>>, f64), ComplexityRefactoringError> {
    let mut best_possibility = None;
    let mut best_score = 0.0;

    // Should be higher than 1, to incentivise equalizing complexity of the two functions
    let equality_preference_factor = 1.35;

    for possibility in possibilities {
        let ComplexitiesPrediction {
            removed_complexity,
            new_function_complexity,
        } = get_resulting_complexities(possibility.clone(), syntax_tree, text_content)?;

        let remaining_complexity =
            (original_complexity.clone() - removed_complexity).get_total_complexity();

        let score = original_complexity.get_total_complexity() as f64
            - get_p_norm(
                remaining_complexity as f64,
                new_function_complexity.get_total_complexity() as f64,
                equality_preference_factor,
            );

        println!("{:?}, {}", possibility, score);
        if score > best_score {
            best_possibility = Some(possibility);
            best_score = score;
        }
    }
    Ok((best_possibility, best_score))
}

fn get_p_norm(x: f64, y: f64, exponent: f64) -> f64 {
    f64::powf(
        f64::powf(x, exponent) + f64::powf(y, exponent),
        1.0 / exponent,
    )
}

fn evaluate_node<'a>(
    node: Node<'a>,
    text_content: &XcodeText,
    syntax_tree: &'a SwiftSyntaxTree,
    scopes: &Vec<Scope>,
) -> Result<Vec<PossibleMethodExtraction<'a>>, ComplexityRefactoringError> {
    let mut scopes: Vec<Scope> = scopes.clone();

    let mut possible_extractions: Vec<PossibleMethodExtraction<'a>> = Vec::new();
    for child in node.named_children(&mut node.walk()) {
        if node_children_are_candidates_for_extraction(&child) {
            possible_extractions.push(PossibleMethodExtraction {
                nodes: child.named_children(&mut child.walk()).collect(),
                outside_references: Vec::new(),
            });
        }
        if node_has_own_scope(&child) {
            scopes.push(Scope {
                variables: Vec::new(),
                node: child,
            })
        }
        if let Some(name) = get_variable_name_if_declaration(&child, &text_content) {
            scopes
                .iter_mut()
                .last()
                .expect("Scopes should not be empty here")
                .variables
                .push(name);
        }
        /*
        if child.kind() == "simple_identifier" {
            println!(
                "{:?}, {:?}",
                get_node_text(&child, text_content).unwrap(),
                node.kind()
            );
        }*/
        possible_extractions.append(&mut evaluate_node(
            child,
            text_content,
            syntax_tree,
            &mut scopes,
        )?);
    }
    Ok(possible_extractions)
}

fn node_children_are_candidates_for_extraction(node: &Node) -> bool {
    node.kind() == "statements" // Restricting to blocks for now
}

fn node_has_own_scope(node: &Node) -> bool {
    node.kind() == "statements" // TODO: Is this true??
}

// We need to track which variables were declared within each scope, because global variables should be ignored (and can't be found).
fn get_variable_name_if_declaration(node: &Node, text_content: &XcodeText) -> Option<XcodeText> {
    match node.kind() {
        "property_declaration" => {
            let name_node = node
                .child_by_field_name("name")?
                .child_by_field_name("bound_identifier")?;
            return get_node_text(&name_node, &text_content).ok();
        }
        "function_declaration" => {
            // TODO
        }
        "parameter" => {
            for child in node.children_by_field_name("name", &mut node.walk()) {
                if child.kind() == "simple_identifier" {
                    return get_node_text(&child, &text_content).ok();
                }
            }
        }
        "for_statement" => {
            let name_node = node
                .child_by_field_name("item")?
                .child_by_field_name("bound_identifier")?;

            // There can be cases where we have an ERROR node etc., so just return None if no name is found
            // TODO: Should we handle this differently? Maybe don't check for method extraction if an error is contained in the node. Then treat this as a real error if the assertion of simple_identifier fails.
            if name_node.kind() == "simple_identifier" {
                return get_node_text(&name_node, &text_content).ok();
            }
        }
        _ => (),
    }
    // TODO: Fill in other cases.
    return None;
}

struct ComplexitiesPrediction {
    removed_complexity: Complexities,
    new_function_complexity: Complexities,
}

fn get_resulting_complexities(
    method_extraction: PossibleMethodExtraction,
    syntax_tree: &SwiftSyntaxTree,
    text_content: &XcodeText,
) -> Result<ComplexitiesPrediction, ComplexityRefactoringError> {
    let removed_complexity = method_extraction.nodes.iter().try_fold(
        Complexities::new(),
        |acc, n| -> Result<Complexities, ComplexityRefactoringError> {
            Ok(acc
                + syntax_tree
                    .get_node_metadata(n)
                    .map_err(|e| ComplexityRefactoringError::GenericError(e.into()))?
                    .complexities
                    .clone())
        },
    )?;

    let mut new_function_complexity = Complexities::new();

    for node in method_extraction.nodes {
        // Start depth at 1, since we assume wrapping nodes in a function_declaration
        new_function_complexity +=
            calculate_cognitive_complexities(&node, &text_content, &mut HashMap::new(), Some(1))
                .map_err(|e| ComplexityRefactoringError::GenericError(e.into()))?
    }

    return Ok(ComplexitiesPrediction {
        removed_complexity,
        new_function_complexity,
    });
}

#[cfg(test)]
mod tests {
    mod method_extraction {

        use crate::core_engine::{
            features::complexity_refactoring::check_for_method_extraction,
            syntax_tree::SwiftSyntaxTree, XcodeText,
        };
        // TODO: Add nullish coalescing operator to complexity
        #[test]
        fn makes_correct_suggestion() {
            let text_content = XcodeText::from_str(
                r#"
                public func extractName(input: String) -> String {
                    if input is String {                                // + 1
                        let start = String(input.prefix(1))
                        let end = String(input.suffix(1));
                        var result = start + end;
                        return result;
                    } else if input is Int {                            // + 1
                        let result: Int;
                        if (Int(input) ?? 0 < 1) {                      // + 2 (1 for nesting)
                            result = 0;
                        }
                        var a = 0;
                        var b = 1;
                        for i in 1..<(Int(input) ?? 0) {                // + 2 (1 for nesting)
                            let c = a + b;
                            a = b;
                            b = c;
                        }
                        result = c;
                        result = b;
                        return String(b);
                    } else {                                            // + 1
                        return "undefined";
                    }
                }
            "#,
            );

            let mut swift_syntax_tree = SwiftSyntaxTree::new();
            swift_syntax_tree.parse(&text_content).unwrap();
            let tree = swift_syntax_tree.tree().unwrap();
            let root_node = tree.root_node();

            let expected_node_kinds = vec![
                "property_declaration",
                "if_statement",
                "property_declaration",
                "property_declaration",
                "for_statement",
                "assignment",
                "assignment",
                "control_transfer_statement",
            ];

            let result =
                check_for_method_extraction(root_node, &text_content, &swift_syntax_tree).unwrap();

            assert_eq!(
                result
                    .unwrap()
                    .iter()
                    .map(|n| n.kind().to_string())
                    .collect::<Vec<String>>(),
                expected_node_kinds
            );
        }
    }
}
