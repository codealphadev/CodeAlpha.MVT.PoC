// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.

export interface FERefactoringSuggestion {
  id: string;
  new_text_content_string: string;
  old_text_content_string: string;
  new_complexity: number;
  prev_complexity: number;
  main_function_name: string | null;
}