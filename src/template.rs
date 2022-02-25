use crate::types::{Template, TemplateComponent};

use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::PathBuf;

impl Template {
    pub fn new<P>(path: P) -> Result<Template, String>
    where
        P: Into<PathBuf>,
    {
        Self::get_template(path)
    }

    fn get_template<P>(path: P) -> Result<Template, String>
    where
        P: Into<PathBuf>,
    {
        let path = PathBuf::from(path.into());
        let template_file = if path.exists() && path.is_file() {
            read_to_string(path).map_err(|e| e.to_string())?
        } else {
            return Err("Path either doesnt exist or is a directory".into());
        };

        Ok(Template {
            template: Self::parse_template(template_file)?,
        })
    }

    pub fn create_from_string<S>(content: S) -> Result<Template, String>
    where
        S: Into<String>,
    {
        Ok(Template {
            template: Self::parse_template(content.into())?,
        })
    }

    fn parse_template(content: String) -> Result<Vec<TemplateComponent>, String> {
        let mut components = Vec::new();
        let mut chars = content.chars().peekable();
        let mut curr_token = String::new();
        let mut is_template_text = true;
        while let Some(chr) = chars.next() {
            match chr {
                '{' => {
                    if !is_template_text {
                        return Err("Cannot have a curly brace in variable name".into());
                    }

                    match chars.peek() {
                        Some('{') => {
                            if curr_token.ends_with('\\') {
                                // escaped
                                curr_token.pop();
                                curr_token.push('{');
                            } else {
                                if curr_token.len() != 0 {
                                    components
                                        .push(TemplateComponent::TemplatePart(curr_token.clone()));
                                }
                                curr_token.clear();
                                chars.next();
                                is_template_text = false;
                            }
                        }
                        Some(chr) => {
                            curr_token.push('{');
                            curr_token.push(*chr);
                            chars.next();
                        }

                        None => curr_token.push('{'),
                    }
                }
                '}' => {
                    if is_template_text {
                        curr_token.push('}');
                    } else {
                        match chars.next() {
                            Some('}') => {
                                if curr_token.len() == 0 {
                                    return Err("Cannot have empty variable name".into());
                                } else {
                                    components
                                        .push(TemplateComponent::InputPart(curr_token.clone()));
                                    curr_token.clear();
                                    is_template_text = true;
                                }
                            }
                            Some(_) => {
                                return Err("Cannot have a curly brace in variable name".into())
                            }
                            None => return Err("Unexpected end of input".into()),
                        }
                    }
                }
                x if x.is_alphabetic() || x == '_' => {
                    curr_token.push(chr);
                }

                _ => {
                    if is_template_text {
                        curr_token.push(chr);
                    } else {
                        if !chr.is_whitespace() {
                            return Err(format!("unexpected char '{}'", chr));
                        }
                    }
                }
            }
        }

        if is_template_text {
            if curr_token.len() != 0 {
                components.push(TemplateComponent::TemplatePart(curr_token));
            }
        } else {
            return Err("Unterminated '{'".into());
        }

        Ok(components)
    }

    pub fn soak(&self, vars: HashMap<String, String>) -> Result<String, String> {
        let mut soaked_template = String::new();
        for component in self.template.iter() {
            soaked_template.push_str(match component {
                TemplateComponent::TemplatePart(string) => string,
                TemplateComponent::InputPart(varname) => match vars.get(varname) {
                    Some(value) => value,
                    None => return Err(format!("Missing variable `{varname}`")),
                },
            });
        }

        Ok(soaked_template)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::template_vars;

    #[test]
    fn it_parses_template() {
        assert_eq!(
            Template::create_from_string("Hello there, {{ name }}! This is a working template!"),
            Ok(Template {
                template: vec![
                    TemplateComponent::TemplatePart("Hello there, ".into()),
                    TemplateComponent::InputPart("name".into()),
                    TemplateComponent::TemplatePart("! This is a working template!".into()),
                ]
            })
        );
        assert_eq!(
            Template::create_from_string("Hello there,  name }! This is a working template!"),
            Ok(Template {
                template: vec![TemplateComponent::TemplatePart(
                    "Hello there,  name }! This is a working template!".into()
                ),]
            })
        );
        assert_eq!(
            Template::create_from_string(
                "Hello there, {{ name }}! I can have {{n}} more variables!"
            ),
            Ok(Template {
                template: vec![
                    TemplateComponent::TemplatePart("Hello there, ".into()),
                    TemplateComponent::InputPart("name".into()),
                    TemplateComponent::TemplatePart("! I can have ".into()),
                    TemplateComponent::InputPart("n".into()),
                    TemplateComponent::TemplatePart(" more variables!".into()),
                ]
            })
        );
        assert_eq!(
            Template::create_from_string("{{_}}"),
            Ok(Template {
                template: vec![TemplateComponent::InputPart("_".into()),]
            })
        );
        assert_eq!(
            Template::create_from_string("double open bracket but it's escaped \\{{{_}}"),
            Ok(Template {
                template: vec![
                    TemplateComponent::TemplatePart(
                        "double open bracket but it's escaped {".into()
                    ),
                    TemplateComponent::InputPart("_".into()),
                ]
            })
        );
        assert_eq!(
            Template::create_from_string("Hello there, { name }! This is a working template!"),
            Ok(Template {
                template: vec![TemplateComponent::TemplatePart(
                    "Hello there, { name }! This is a working template!".into()
                ),]
            })
        );
        assert_eq!(
            Template::create_from_string("Hello there, {{ name }}! I can have {n} more variables!"),
            Ok(Template {
                template: vec![
                    TemplateComponent::TemplatePart("Hello there, ".into()),
                    TemplateComponent::InputPart("name".into()),
                    TemplateComponent::TemplatePart("! I can have {n} more variables!".into()),
                ]
            })
        );
        assert_eq!(
            Template::create_from_string("{_}"),
            Ok(Template {
                template: vec![TemplateComponent::TemplatePart("{_}".into()),]
            })
        );
        assert_eq!(
            Template::create_from_string("more escaping  \\{{_}}"),
            Ok(Template {
                template: vec![TemplateComponent::TemplatePart(
                    "more escaping  {{_}}".into()
                )]
            })
        );
        assert_eq!(
            Template::create_from_string("Hello there, {{ name}}}! This is a working template!"),
            Ok(Template {
                template: vec![
                    TemplateComponent::TemplatePart("Hello there, ".into()),
                    TemplateComponent::InputPart("name".into()),
                    TemplateComponent::TemplatePart("}! This is a working template!".into()),
                ]
            })
        );
    }
    #[test]
    fn it_fails_templating() {
        assert_eq!(
            Template::create_from_string("Hello there, {{{ name }}! This is a working template!"),
            Err("Cannot have a curly brace in variable name".into())
        );
        assert_eq!(
            Template::create_from_string(
                "Hello there, {{ name}}}! This is a working template! {{ another_var }"
            ),
            Err("Unexpected end of input".into())
        );
        assert_eq!(
            Template::create_from_string("Hello there, {{ }}! This is a working template!"),
            Err("Cannot have empty variable name".into())
        );
        assert_eq!(
            Template::create_from_string("Hello there, {{ -}}! This is a working template!"),
            Err("unexpected char '-'".into())
        );
    }

    #[test]
    fn it_soaks_template() {
        let template =
            Template::create_from_string("Hello there, {{ name }}! This is a working template!")
                .unwrap();

        assert_eq!(
            template.soak(template_vars! {"name" => "Template Monster"}),
            Ok("Hello there, Template Monster! This is a working template!".into())
        );
    }
    #[test]
    fn it_fails_soaking() {
        let template =
            Template::create_from_string("Hello there, {{ name }}! This is a working template!")
                .unwrap();

        assert_eq!(
            template.soak(template_vars! {"nam" => "Template Monster"}),
            Err("Missing variable `name`".into())
        );
    }
}
