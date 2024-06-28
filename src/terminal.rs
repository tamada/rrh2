pub fn to_string_in_columns(values: Vec<String>) -> String {
    match termion::terminal_size() {
        Ok((cols, _)) => to_string_in_column_with(values, cols as usize, 2),
        Err(_) => to_string_in_column_with(values, 80 as usize, 2),
    }
}

fn find_max_length_and_column_count(
    values: &Vec<String>,
    cols: usize,
    spacer: usize,
) -> (usize, usize) {
    let mut max = 0;
    for v in values {
        if v.len() > max {
            max = v.len();
        }
    }
    let column_count = (cols + spacer) / (max as usize + spacer);
    (max, column_count)
}

fn padding(values: Vec<String>, size: usize) -> Vec<String> {
    values
        .iter()
        .map(|v| {
            let mut s = v.clone();
            for _ in 0..(size - v.len()) {
                s.push(' ');
            }
            s
        })
        .collect()
}

pub fn to_string_in_column_with(values: Vec<String>, cols: usize, spacer: usize) -> String {
    let (max, column_count) = find_max_length_and_column_count(&values, cols, spacer);
    let padding_values = padding(values, max);
    let mut lines = Vec::new();
    let mut line = Vec::<u8>::new();
    let space = (0..spacer).into_iter().map(|_| " ").collect::<String>();
    for (i, item) in padding_values.iter().enumerate() {
        line.extend(item.as_bytes());
        if i % column_count == (column_count - 1) || i == padding_values.len() - 1 {
            let r = String::from_utf8(line.clone()).unwrap();
            lines.push(r);
            line.clear();
        } else {
            line.extend(space.as_bytes())
        }
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_in_columns_with() {
        let v = vec![
            "macOS",
            "Linux",
            "Windows",
            "Go",
            "VisualStudioCode",
            "JetBrains",
        ];
        let v1: Vec<String> = v.iter().map(|s| s.to_string()).collect();
        let r1 = to_string_in_column_with(v1.clone(), 125, 1);
        assert_eq!(r1.trim(), "macOS            Linux            Windows          Go               VisualStudioCode JetBrains");

        let r2 = to_string_in_column_with(v1.clone(), 125, 2);
        assert_eq!(r2.trim(), "macOS             Linux             Windows           Go                VisualStudioCode  JetBrains");
    }
}
