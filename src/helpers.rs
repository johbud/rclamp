pub const EXPLORER: &str = "explorer";
pub const FINDER: &str = "finder";
pub const PROJECT_FILE_NAME: &str = "project.yaml";

pub fn sanitize_string(mut s: String) -> String {
    let mut output = String::new();
    s = s.to_lowercase();
    for c in s.chars() {
        let mut cc = c.clone();
        if cc.is_ascii_alphanumeric() {
            output.push(cc);
        } else {
            cc = match cc {
                '_' => '_',
                '-' => '_',
                'å' => 'a',
                'ä' => 'a',
                'ö' => 'o',
                _ => continue,
            };
            output.push(cc);
        }
    }

    output
}
