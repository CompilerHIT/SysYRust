pub fn preprocess(input: &str) -> String {
    let mut line_number = 1;
    let mut result = String::new();

    for line in input.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        for part in parts {
            let pparts: Vec<&str> = part.split(";").collect();
            for i in 0..pparts.len() - 1 {
                if pparts[i].contains("starttime()") {
                    result.push_str(&format!("_sysy_starttime({});", line_number));
                } else if pparts[i].contains("stoptime()") {
                    result.push_str(&format!("_sysy_stoptime({});", line_number));
                } else {
                    result.push_str((" ".to_string() + pparts[i] + ";").as_str());
                }
            }
            if !pparts[pparts.len() - 1].is_empty() {
                if pparts[pparts.len() - 1].contains("starttime()") {
                    result.push_str(&format!("_sysy_starttime({})", line_number));
                } else if pparts[pparts.len() - 1].contains("stoptime()") {
                    result.push_str(&format!("_sysy_stoptime({})", line_number));
                } else {
                    result.push_str((" ".to_string() + pparts[pparts.len() - 1]).as_str());
                }
            }
        }
        result.push('\n');
        line_number += 1;
    }

    result
}

#[cfg(test)]
mod pre_test {
    use crate::frontend::preprocess::preprocess;

    #[test]
    fn test() {
        let input = r#"
            This is some code.
            starttime();stoptime();stoptime();stoptime()    ;
            More code here.
            starttime();
            Some more code.
        "#;

        let processed = preprocess(input);
        println!("{}", processed);
    }
}
