use tracing::{debug, error};

use crate::{
    core_engine::{
        features::{
            complexity_refactoring::{ComplexityRefactoringError, SuggestionsArcMutex},
            ComplexityRefactoring, FeatureKind, SuggestionId,
        },
        CodeDocument, XcodeText,
    },
    platform::macos::{replace_text_content, set_selected_text_range, GetVia},
};

pub async fn perform_suggestion(
    code_document: CodeDocument,
    suggestion_id: SuggestionId,
    suggestions_arc: SuggestionsArcMutex,
) -> Result<(), ComplexityRefactoringError> {
    let window_uid = code_document.editor_window_props().window_uid;
    let suggestions =
        ComplexityRefactoring::get_suggestions_for_window(suggestions_arc.clone(), window_uid);

    let suggestion_to_apply = suggestions
        .get(&suggestion_id)
        .ok_or(ComplexityRefactoringError::SuggestionNotFound(
            suggestion_id.to_string(),
        ))?
        .clone();

    debug!(
        ?suggestion_to_apply,
        feature = FeatureKind::ComplexityRefactoring.to_string(),
        "Performing suggestion"
    );

    let text_range_to_scroll_to_after_performing =
        ComplexityRefactoring::get_text_position_to_scroll_to_after_performing(
            &suggestion_to_apply,
        );

    let new_content = suggestion_to_apply.clone().new_text_content_string.ok_or(
        ComplexityRefactoringError::SuggestionIncomplete(suggestion_to_apply),
    )?;

    let text_content = code_document
        .text_content()
        .ok_or(ComplexityRefactoringError::InsufficientContext)?;

    match replace_text_content(
        &text_content,
        &XcodeText::from_str(&new_content),
        code_document.selected_text_range(),
    )
    .await
    {
        Ok(_) => {}
        Err(err) => {
            error!(?err, "Error replacing text content");
            return Err(ComplexityRefactoringError::GenericError(err.into()));
        }
    }

    ComplexityRefactoring::remove_suggestion_and_publish(
        &window_uid,
        &suggestion_id,
        &suggestions_arc,
    )?;

    match text_range_to_scroll_to_after_performing {
        Ok(range) => {
            _ = set_selected_text_range(&range, &GetVia::Current);
        }
        Err(e) => {
            error!(
                ?e,
                "Error getting final cursor position after performing suggestion"
            );
            return Err(e);
        }
    }

    Ok(())
}
