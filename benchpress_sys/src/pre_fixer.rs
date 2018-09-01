use regex::{Regex, Captures};
use onig;

// `<!-- BEGIN stuff -->` => `<!-- BEGIN ../stuff -->` and `<!-- BEGIN stuff -->`
// we need to add the fallback by duplicating under a different key
// only apply to nested blocks
fn fix_iter(input: &str, first: bool) -> String {
    lazy_static! {
        static ref LEGACY_ITER_PATTERN: onig::Regex = onig::Regex::new(r"<!-- BEGIN ([^./][@a-zA-Z0-9/.\-_:]+?) -->([\s\S]*?)<!-- END \1 -->").unwrap();
    }
    
    LEGACY_ITER_PATTERN.replace_all(input, |caps: &onig::Captures| {
        let subject = caps.at(1).unwrap_or("");
        let body = fix_iter(caps.at(2).unwrap_or(""), false);

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
fn combined(input: &str) -> String {
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

    COMBINED.replace_all(input, |caps: &Captures| {
        if caps.name("if_helpers").is_some() {
            // add root data to legacy if helpers
            let name = &caps["if_helpers_name"].to_string();
            let args = &caps["if_helpers_args"].to_string();

            if args.is_empty() {
                format!("<!-- IF function.{}, @root -->", name)
            } else {
                format!("<!-- IF function.{}, @root, {} -->", name, args)
            }
        } else if caps.name("loop_helpers").is_some() {
            // add value context for in-loop helpers
            let name = &caps["loop_helpers_name"].to_string();
            format!("{{function.{}, @value}}", name)
        } else if caps.name("outside_tokens").is_some() {
            // wrap `@key`, `@value`, `@index` in mustaches
            // if they aren't in a mustache already
            let orig = &caps[0];
            
            if let Some(lone) = caps.name("outside_tokens_lone") {
                format!("{{{}}}", lone.as_str())
            } else {
                orig.to_string()
            }
        } else {
            String::new()
        }
    }).into_owned()
}

pub fn pre_fix(input: &str) -> String {
    combined(&fix_iter(input, true))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loop_helpers() {
        let source = "
        {function.foo_bar}
        ";
        let expected = "
        {function.foo_bar, @value}
        ";

        assert_eq!(combined(source), expected);
    }

    #[test]
    fn outside_tokens() {
        let source = "
        @key : @value
        @index : @value
        {@key} : {@value}
        {@index} : {@value}
        ";
        let expected = "
        {@key} : {@value}
        {@index} : {@value}
        {@key} : {@value}
        {@index} : {@value}
        ";

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
        ";
        let expected = "
        <!-- IF function.foo_bar, @root -->
        asdf ghjk
        <!-- END -->

        <!-- IF function.hello_world, @root, one, two -->
        qwer tyui
        <!-- END -->
        ";

        assert_eq!(combined(source), expected);
    }
}
