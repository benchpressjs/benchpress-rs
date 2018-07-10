use regex::{Regex, Captures};
use onig;

// `<!-- BEGIN stuff -->` => `<!-- BEGIN ../stuff -->` and `<!-- BEGIN stuff -->`
// we need to add the fallback by duplicating under a different key
// only apply to nested blocks
fn fix_iter(input: String, first: bool) -> String {
    lazy_static! {
        static ref LEGACY_ITER_PATTERN: onig::Regex = onig::Regex::new(r"<!-- BEGIN ([^./][@a-zA-Z0-9/.\-_:]+?) -->([\s\S]*?)<!-- END \1 -->").unwrap();
    }
    
    LEGACY_ITER_PATTERN.replace_all(input.as_ref(), |caps: &onig::Captures| {
        let subject = caps.at(1).unwrap_or("");
        let body = fix_iter(caps.at(2).unwrap_or("").to_string(), false);

        if first {
            format!("<!-- BEGIN {} -->{}<!-- END {} -->", subject, body, subject)
        } else {
            format!(
                "<!-- IF ../{} --><!-- BEGIN ../{} -->{}<!-- END ../{} --><!-- ELSE --><!-- BEGIN {} -->{}<!-- END {} --><!-- ENDIF ../{} -->",
                subject, subject, body, subject, subject, body, subject, subject
            )
        }
    })
}

// combined regex replacement
fn combined(input: String) -> String {
    lazy_static! {
        static ref COMBINED: Regex = Regex::new(r"(?x)
            # helpers root
            (?P<if_helpers>
                # '\x20' is a space
                <!--\x20IF\x20(
                    ?:function\.
                    (?P<if_helpers_name>[@a-zA-Z0-9/._:]+)
                    (?:\s*,\s*)?
                    (?P<if_helpers_args>.*?)
                )\x20-->
            )
            |
            # legacy loop helpers
            (?P<loop_helpers>
                \{function\.(?P<loop_helpers_name>[^}\n\x20,]+)\}
            )
            |
            # outside tokens
            (?P<outside_tokens>
                (?:\{{1,2}[^}]+\}{1,2})|(?:<!--[^>]+-->)|
                (?P<outside_tokens_lone>@key|@value|@index)
            )
        ").unwrap();
    }

    COMBINED.replace_all(input.as_ref(), |caps: &Captures| {
        if let Some(_) = caps.name("if_helpers") {
            // add root data to legacy if helpers
            let name = String::from(&caps["if_helpers_name"]);
            let args = String::from(&caps["if_helpers_args"]);

            if args.len() > 0 {
                format!("<!-- IF function.{}, @root, {} -->", name, args)
            } else {
                format!("<!-- IF function.{}, @root -->", name)
            }
        } else if let Some(_) = caps.name("loop_helpers") {
            // add value context for in-loop helpers
            let name = String::from(&caps["loop_helpers_name"]);
            format!("{{function.{}, @value}}", name)
        } else if let Some(_) = caps.name("outside_tokens") {
            // wrap `@key`, `@value`, `@index` in mustaches
            // if they aren't in a mustache already
            let orig = String::from(&caps[0]);
            
            if let Some(lone) = caps.name("outside_tokens_lone") {
                format!("{{{}}}", lone.as_str())
            } else {
                orig
            }
        } else {
            String::new()
        }
    }).into_owned()
}

pub fn pre_fix(input: String) -> String {
    combined(fix_iter(input, true))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loop_helpers() {
        let source = "
        {function.foo_bar}
        ".to_string();
        let expected = "
        {function.foo_bar, @value}
        ".to_string();

        assert_eq!(combined(source), expected);
    }

    #[test]
    fn outside_tokens() {
        let source = "
        @key : @value
        @index : @value
        {@key} : {@value}
        {@index} : {@value}
        ".to_string();
        let expected = "
        {@key} : {@value}
        {@index} : {@value}
        {@key} : {@value}
        {@index} : {@value}
        ".to_string();

        assert_eq!(combined(source), expected);
    }

    #[test]
    fn helpers_root() {
        let source = "
        <!-- IF function.foo_bar -->
        asdf ghjk
        <!-- END -->

        <!-- IF function.hello_world, one, two -->
        qwer tyui
        <!-- END -->
        ".to_string();
        let expected = "
        <!-- IF function.foo_bar, @root -->
        asdf ghjk
        <!-- END -->

        <!-- IF function.hello_world, @root, one, two -->
        qwer tyui
        <!-- END -->
        ".to_string();

        assert_eq!(combined(source), expected);
    }
}
