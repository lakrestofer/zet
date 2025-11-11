use deunicode::deunicode_char;

/// slugify functiont that preserves '/' character and '.' character
/// other characters gets turned into their ascii equivalent or the '-' character
pub fn slugify<S: AsRef<str>>(s: S) -> String {
    slugify_impl(s.as_ref())
}

fn slugify_impl(s: &str) -> String {
    let mut slug = String::with_capacity(s.len());
    {
        let mut push_char = |x: u8| match x {
            b'a'..=b'z' | b'0'..=b'9' => slug.push(x.into()),
            b'A'..=b'Z' => slug.push((x - b'A' + b'a').into()),
            b'.' => slug.push('.'),
            b'/' => slug.push('/'),
            _ => slug.push('-'),
        };

        for c in s.chars() {
            if c.is_ascii() {
                (push_char)(c as u8);
            } else {
                for &cx in deunicode_char(c).unwrap_or("-").as_bytes() {
                    (push_char)(cx);
                }
            }
        }
    }

    if slug.ends_with('-') {
        slug.pop();
    }
    // We likely reserved more space than needed.
    slug.shrink_to_fit();
    slug
}
