
const ESCAPE_CHARS: [char; 2] = [
    'n',
    'e',
];


pub fn string(unescaped: &str) -> String {
    let mut index = 0;
    let mut escaped = String::new();

    let characters = unescaped.chars().collect::<Vec<char>>();

    while index < characters.len() {
        if characters[index] == '\\' && index + 1 != characters.len() {
            index += 1;
            if ESCAPE_CHARS.contains(&characters[index]) {
                escaped.push(match characters[index] {
                    'n' => '\n',
                    'e' => '\x1B',
                    _ => unreachable!(),
                });
                index += 1;
                continue;
            } else {
                // if this isnt here "\<Unknown escape>" will result in "<Unknown escape>"
                escaped.push('\\');
            }
        }
        escaped.push(characters[index]);
        index += 1;
    }

    escaped
}


