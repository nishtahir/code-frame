use std::{cmp::max, collections::HashMap};

pub struct NodeLocation {
    pub start: Location,
    pub end: Location,
}

#[derive(Debug)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}
pub struct CodeFrameOptions {
    pub lines_above: usize,
    pub lines_below: usize,
}

type LineMarkers =
    HashMap</* line_number: */ usize, (/* start_col: */ usize, /* len: */ usize)>;

#[derive(Debug)]
pub struct MarkerLines {
    pub start: usize,
    pub end: usize,
    pub marker_lines: LineMarkers,
}

fn marker_lines(lines: &[&str], loc: NodeLocation, options: CodeFrameOptions) -> MarkerLines {
    let lines_above = options.lines_above;
    let lines_below = options.lines_below;

    // Note: These are not 0-indexed
    let start_line = loc.start.line;
    let start_col = loc.start.column;
    let end_line = loc.end.line;
    let end_col = loc.end.column;

    let start = ((start_line as i32) - (lines_above as i32)).max(0) as usize;
    let end = lines.len().min(end_line + lines_below).max(0);
    let line_diff = end_line - start_line;

    let mut marker_lines: LineMarkers = HashMap::new();

    if line_diff > 0 {
        // The marker spans multiple lines
        for i in 0..=line_diff {
            let line_number = i + start_line;
            if i == 0 {
                // The first line
                marker_lines.insert(
                    line_number,
                    (start_col, lines[line_number].len() - start_col),
                );
            } else if i == line_diff {
                // The last line
                marker_lines.insert(line_number, (0, end_col));
            } else {
                // A line in the middle
                marker_lines.insert(line_number, (0, lines[line_number].len()));
            }
        }
    } else {
        // The marker is on a single line
        if start_col == end_col {
            // The marker is a single character
            marker_lines.insert(start_line, (start_col, 0));
        } else {
            // The marker is a range of characters
            marker_lines.insert(start_line, (start_col, end_col - start_col));
        }
    }

    MarkerLines {
        start,
        end,
        marker_lines,
    }
}

pub fn code_frame(lines: &[&str], loc: NodeLocation, context_window: CodeFrameOptions) -> String {
    let marker_lines = marker_lines(lines, loc, context_window);

    let max_line_number_width = marker_lines.end.to_string().len() + 1;
    let context = &lines[marker_lines.start..marker_lines.end];

    context
        .iter()
        .enumerate()
        .map(|(i, line)| {
            // Adjust the line number to be 1-indexed
            let line_number = i + marker_lines.start;
            let line_number_width = line_number.to_string().len();
            let line_number_padding = max_line_number_width - line_number_width;
            let line_number_padding = " ".repeat(line_number_padding);

            let mut marker_line = None;
            if let Some((start_col, len)) = marker_lines.marker_lines.get(&line_number) {
                // We're marking at least 1 character
                let marker = "^".repeat(max(*len, 1));

                // Pad the marker line with spaces to align with the start column
                let marker_padding = " ".repeat(*start_col);
                marker_line = Some(format!(
                    "{}{}{}",
                    marker_padding, marker, line_number_padding
                ));
            }

            if let Some(marker_line) = marker_line {
                // Add a > to the start of the line if it's a marked line
                return format!(
                    "> {} | {}\n  {} | {}",
                    line_number, line, line_number_padding, marker_line
                );
            }
            // Otherwise, just print the line number and line
            format!("  {} | {}", line_number, line)
        })
        .collect::<Vec<String>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_frame() {
        let code = r#"println!("Hello, world!")"#;
        let lines = code.trim().lines().collect::<Vec<_>>();

        let loc = NodeLocation {
            start: Location { line: 0, column: 0 },
            end: Location { line: 0, column: 7 },
        };

        let context_window = CodeFrameOptions {
            lines_above: 3,
            lines_below: 3,
        };

        let res = code_frame(&lines, loc, context_window);
        insta::assert_yaml_snapshot!(res)
    }


    #[test]
    fn test_code_frame_no_content() {
        let code = r#"println!("Hello, world!")"#;
        let lines = code.trim().lines().collect::<Vec<_>>();

        let loc = NodeLocation {
            start: Location { line: 0, column: 0 },
            end: Location { line: 0, column: 0 },
        };

        let context_window = CodeFrameOptions {
            lines_above: 0,
            lines_below: 0,
        };

        let res = code_frame(&lines, loc, context_window);
        insta::assert_yaml_snapshot!(res)
    }

    #[test]
    fn test_code_frame_long_context() {
        let code = r#"
        // test 1 
        // test 2
        fn main() {
            println!("Hello, world!");
            println!("Hello, world!");
        }
        // test 3
        // test 4
        // test 5
            "#;
        let lines = code.trim().lines().collect::<Vec<_>>();

        let loc = NodeLocation {
            start: Location { line: 3, column: 4 },
            end: Location {
                line: 4,
                column: 11,
            },
        };

        let context_window = CodeFrameOptions {
            lines_above: 10,
            lines_below: 10,
        };

        let res = code_frame(&lines, loc, context_window);
        insta::assert_yaml_snapshot!(res)
    }
}
