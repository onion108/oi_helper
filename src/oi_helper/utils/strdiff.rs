//! This file contains some utility functions that will output the difference between two strings.

use crossterm::style::{Stylize, StyledContent};

pub fn colored_diff(original: &str, target: &str) -> (Vec<StyledContent<String>>, Vec<StyledContent<String>>) {
    let original_chars = original.chars().collect::<Vec<char>>();
    let target_chars = target.chars().collect::<Vec<char>>();
    let mut result_original: Vec<StyledContent<String>> = vec![];
    let mut result_target: Vec<StyledContent<String>> = vec![];

    // If the lengths of two are the same.
    if original_chars.len() == target_chars.len() {

        // Iterate over each characters and color it.
        for i in 0..target_chars.len() {
            result_original.push(format!("{}", original_chars[i]).green());
            result_target.push(if original_chars[i] == target_chars[i] {
                format!("{}", target_chars[i]).green()
            } else {
                format!("{}", target_chars[i]).on_red().bold()
            });
        }
    } else {
        if original_chars.len() < target_chars.len() {
            // The target is longer.
            let mut last_counter = 0;
            for i in 0..original_chars.len() {
                last_counter = i+1;
                result_original.push(format!("{}", original_chars[i]).green());
                result_target.push(if original_chars[i] == target_chars[i] {
                    format!("{}", target_chars[i]).green()
                } else {
                    format!("{}", target_chars[i]).on_red().bold()
                });
            }

            // Color the remaining parts.
            while last_counter < target_chars.len() {
                result_target.push(format!("{}", target_chars[last_counter]).on_red().bold());
                last_counter += 1;
            }
        } else {
            // The original is longer.
            let mut last_counter = 0;
            for i in 0..target_chars.len() {
                last_counter = i+1;
                result_original.push(format!("{}", original_chars[i]).green());
                result_target.push(if original_chars[i] == target_chars[i] {
                    format!("{}", target_chars[i]).green()
                } else {
                    format!("{}", target_chars[i]).on_red().bold()
                });
            }

            // Color the remaining parts.
            while last_counter < original_chars.len() {
                result_target.push(format!(" ").on_red().bold());
                result_original.push(format!("{}", original_chars[last_counter]).green());
                last_counter += 1;
            }
        }
    }

    // Return the results.
    (result_original, result_target)
}
