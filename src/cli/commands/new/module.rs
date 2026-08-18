use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
use walkdir::WalkDir;

use crate::{client::MoldXClient, errors::MoldXError, template::Template};

pub fn new_module(client: &MoldXClient, args: Vec<String>) -> Result<()> {
    let (strategy_name, template_name, module_index) = match args.len() {
        2 => (None, None, 1),
        3 => (Some(args[1].clone()), None, 2),
        4 => (Some(args[1].clone()), Some(args[2].clone()), 3),
        _ => return Err(MoldXError::NewModuleUsage.into()),
    };

    let module_path = PathBuf::from(&args[module_index]);
    if module_path.exists() {
        return Err(MoldXError::ModulePathAlreadyExists { path: module_path }.into());
    }

    fs::create_dir_all(&module_path)?;

    if let Some(strategy_name) = strategy_name {
        let strategy = client
            .get_strategy(&strategy_name)
            .ok_or_else(|| MoldXError::StrategyNotFound { name: strategy_name })?;
        let template = select_template(&strategy, template_name.as_deref())?;
        scaffold_template_dir(&template.dir, &module_path)?;
        println!(
            "Scaffolded module {} from {} / {} at {}",
            module_path.file_name().and_then(|name| name.to_str()).unwrap_or("module"),
            strategy.name,
            template.name,
            module_path.display()
        );
    } else {
        println!("Created module at {}", module_path.display());
    }

    Ok(())
}

fn select_template(strategy: &crate::strategy::Strategy, template_name: Option<&str>) -> Result<Template> {
    let templates: Vec<Template> = strategy
        .templates
        .iter()
        .filter(|template| !template.file_names.is_empty())
        .cloned()
        .collect();

    match template_name {
        Some(name) => strategy
            .templates
            .iter()
            .find(|template| template.name == name && !template.file_names.is_empty())
            .cloned()
            .ok_or_else(|| MoldXError::TemplateNotFound { name: name.to_string(), strategy: strategy.name.clone() }.into()),
        None => match templates.as_slice() {
            [template] => Ok(template.clone()),
            [] => Err(MoldXError::NoScaffoldableTemplate { name: strategy.name.clone() }.into()),
            _ => Err(MoldXError::MultipleTemplates { name: strategy.name.clone() }.into()),
        },
    }
}

fn scaffold_template_dir(template_dir: &Path, target: &Path) -> Result<()> {
    for entry in WalkDir::new(template_dir).follow_links(false).into_iter().filter_map(|entry| entry.ok()) {
        if !entry.file_type().is_file() {
            continue;
        }

        let relative = match entry.path().strip_prefix(template_dir) {
            Ok(relative) => relative,
            Err(_) => continue,
        };

        let destination = target.join(relative);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(entry.path(), &destination)?;
    }

    Ok(())
}
