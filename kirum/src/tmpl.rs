use std::path::PathBuf;

use handlebars::{Handlebars, Helper, RenderContext, Output, HelperResult, Renderable, RenderError};
use libkirum::kirum::Lexis;
use anyhow::{Result, Context, anyhow};

/// Render a dictionary from a list of words, and a template
pub fn generate_from_tmpl(rendered_lang: Vec<Lexis>, template_file: String, rhai_files: Option<Vec<String>>) -> Result<String> {
    let mut reg = Handlebars::new();
    reg.register_helper("string_eq", Box::new(string_eq));
    reg.register_template_file("tmpl", &template_file).context(format!("could not add template file {}", template_file))?;
    if let Some(files) = rhai_files{
        for file in files{
            let script_path = PathBuf::from(file.clone());
            let script = script_path.file_stem()
            .ok_or(anyhow!("could not extract script name from file name: {}", file))?.to_str().ok_or(anyhow!("could not convert file name {} to string", file.clone()))?;
            reg.register_script_helper_file(script, file.clone()).context(format!("could not add script helper file {}", file))?;
            debug!("registered script {} as {}", file, script);
        }
    }
    
   let rendered = reg.render("tmpl", &rendered_lang)?;

   Ok(rendered)
}

/// a template helper, defines a handlebars function that compares two strings
fn string_eq<'reg, 'rc>(
    helper: &Helper<'reg, 'rc>,
    r: &'reg Handlebars<'reg>,
    ctx: &'rc handlebars::Context,
    rc: &mut RenderContext<'reg, 'rc>,
    out: &mut dyn Output,
) -> HelperResult {
    let one = helper.param(0).ok_or(RenderError::new("first param in string_eq not found"))?;
    let two = helper.param(1).ok_or(RenderError::new("second param in string_eq not found"))?;
    if one.render() == two.render() {
        helper.template().map(|t|t.render(r, ctx, rc, out));
    } else {
        helper.inverse().map(|t|t.render(r, ctx, rc, out));
    }

    Ok(())
}