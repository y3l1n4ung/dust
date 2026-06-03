use minijinja::Environment;
use serde::Serialize;

/// Renders a compile-time template with strict variable validation.
pub fn render_template<T>(name: &str, source: &'static str, context: T) -> String
where
    T: Serialize,
{
    let mut env = Environment::new();
    env.add_template(name, source)
        .expect("Dust template source must be valid");
    env.get_template(name)
        .expect("Dust template must be registered")
        .render(context)
        .expect("Dust template context must satisfy template variables")
        .trim_end_matches('\n')
        .to_owned()
}
