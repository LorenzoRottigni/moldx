use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
use walkdir::WalkDir;

use crate::{client::MoldXClient, errors::MoldXError, template::Template};

/// Scaffolds a new module directory.
///
/// Accepts `<module-path>`, `<strategy> <module-path>`, or
/// `<strategy> <template> <module-path>`. When a strategy is given, the
/// selected template is copied into the new module directory.
///
/// # Arguments
///
/// * `client` - The initialized MoldX client.
/// * `args` - Raw arguments; see above for the accepted forms.
///
/// # Returns
///
/// Ok once the module directory has been created.
///
/// # Errors
///
/// Returns [`MoldXError::NewModuleUsage`] on malformed arguments,
/// [`MoldXError::ModulePathAlreadyExists`] if the path already exists,
/// [`MoldXError::StrategyNotFound`] if the strategy is unknown, any error
/// raised while selecting a template, and any IO error while scaffolding.
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

/// Selects the template to scaffold a module from.
///
/// Scaffoldable templates are those containing at least one file. With an
/// explicit name, the matching template is returned; otherwise the strategy
/// must expose exactly one scaffoldable template.
///
/// # Arguments
///
/// * `strategy` - The strategy to pick a template from.
/// * `template_name` - Optional explicit template name.
///
/// # Returns
///
/// The selected template.
///
/// # Errors
///
/// Returns [`MoldXError::TemplateNotFound`] if the named template does not
/// exist or contains no files, [`MoldXError::NoScaffoldableTemplate`] if
/// the strategy exposes none, and [`MoldXError::MultipleTemplates`] if it
/// exposes several without an explicit choice.
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

/// Copies all files of a template directory into the target directory.
///
/// Subdirectories are recreated and hidden files are copied as-is.
///
/// # Arguments
///
/// * `template_dir` - The template directory to copy from.
/// * `target` - The module directory to copy into.
///
/// # Returns
///
/// Ok once all files have been copied.
///
/// # Errors
///
/// Returns an error if directories cannot be created or files cannot be
/// copied.
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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    fn make_client(dir: &std::path::Path) -> MoldXClient {
        let moldx_dir = dir.join(".moldx");
        let strategies_dir = moldx_dir.join("strategies");
        fs::create_dir_all(&strategies_dir).unwrap();
        let config = crate::config::MoldXConfig {
            moldx_dir,
            strategies_dir,
            bin_dir_name: "bin".into(),
            template_dir_name: "template".into(),
            templates_dir_name: "templates".into(),
            modules_dir: dir.to_path_buf(),
            max_resolution_depth: 20,
        };
        MoldXClient::new(config).unwrap()
    }

    #[test]
    fn test_new_module_no_strategy() {
        let dir = tempdir().unwrap();
        let client = make_client(dir.path());
        let module_path = dir.path().join("my-module");
        let result = new_module(&client, vec!["module".into(), module_path.to_str().unwrap().into()]);
        assert!(result.is_ok());
        assert!(module_path.exists());
    }

    #[test]
    fn test_new_module_with_strategy() {
        let dir = tempdir().unwrap();
        let tpl_dir = dir.path().join(".moldx/strategies/docker/template");
        fs::create_dir_all(&tpl_dir).unwrap();
        fs::write(tpl_dir.join("Dockerfile"), "FROM scratch").unwrap();
        let client = make_client(dir.path());
        let module_path = dir.path().join("my-docker-module");
        let result = new_module(&client, vec!["module".into(), "docker".into(), module_path.to_str().unwrap().into()]);
        assert!(result.is_ok());
        assert!(module_path.join("Dockerfile").exists());
        let content = fs::read_to_string(module_path.join("Dockerfile")).unwrap();
        assert_eq!(content, "FROM scratch");
    }

    #[test]
    fn test_new_module_path_already_exists() {
        let dir = tempdir().unwrap();
        let client = make_client(dir.path());
        let module_path = dir.path().join("existing");
        fs::create_dir(&module_path).unwrap();
        let result = new_module(&client, vec!["module".into(), module_path.to_str().unwrap().into()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_new_module_strategy_not_found() {
        let dir = tempdir().unwrap();
        let client = make_client(dir.path());
        let module_path = dir.path().join("my-module");
        let result = new_module(&client, vec!["module".into(), "nonexistent".into(), module_path.to_str().unwrap().into()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_new_module_wrong_arg_count() {
        let dir = tempdir().unwrap();
        let client = make_client(dir.path());
        let result = new_module(&client, vec!["module".into()]);
        assert!(result.is_err());
    }

    #[test]
    fn test_select_template_single() {
        let dir = tempdir().unwrap();
        let tpl_dir = dir.path().join("template");
        fs::create_dir(&tpl_dir).unwrap();
        fs::write(tpl_dir.join("Dockerfile"), "").unwrap();
        let strategy = crate::strategy::Strategy::new(
            dir.path().to_path_buf(),
            &crate::config::MoldXConfig {
                moldx_dir: PathBuf::from("/nonexistent"),
                strategies_dir: dir.path().to_path_buf(),
                bin_dir_name: "bin".into(),
                template_dir_name: "template".into(),
                templates_dir_name: "templates".into(),
                modules_dir: PathBuf::from("/nonexistent"),
                max_resolution_depth: 20,
            },
        ).unwrap();
        let result = select_template(&strategy, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_select_template_none_available() {
        let dir = tempdir().unwrap();
        let tpl_dir = dir.path().join("template");
        fs::create_dir(&tpl_dir).unwrap();
        let strategy = crate::strategy::Strategy::new(
            dir.path().to_path_buf(),
            &crate::config::MoldXConfig {
                moldx_dir: PathBuf::from("/nonexistent"),
                strategies_dir: dir.path().to_path_buf(),
                bin_dir_name: "bin".into(),
                template_dir_name: "template".into(),
                templates_dir_name: "templates".into(),
                modules_dir: PathBuf::from("/nonexistent"),
                max_resolution_depth: 20,
            },
        ).unwrap();
        let result = select_template(&strategy, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_select_template_multiple_available() {
        let dir = tempdir().unwrap();
        let templates_dir = dir.path().join("templates");
        let t1 = templates_dir.join("docker");
        let t2 = templates_dir.join("rust");
        fs::create_dir_all(&t1).unwrap();
        fs::create_dir_all(&t2).unwrap();
        fs::write(t1.join("Dockerfile"), "").unwrap();
        fs::write(t2.join("Cargo.toml"), "").unwrap();
        let strategy = crate::strategy::Strategy::new(
            dir.path().to_path_buf(),
            &crate::config::MoldXConfig {
                moldx_dir: PathBuf::from("/nonexistent"),
                strategies_dir: dir.path().to_path_buf(),
                bin_dir_name: "bin".into(),
                template_dir_name: "template".into(),
                templates_dir_name: "templates".into(),
                modules_dir: PathBuf::from("/nonexistent"),
                max_resolution_depth: 20,
            },
        ).unwrap();
        let result = select_template(&strategy, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_select_template_by_name() {
        let dir = tempdir().unwrap();
        let templates_dir = dir.path().join("templates");
        let t1 = templates_dir.join("docker");
        let t2 = templates_dir.join("rust");
        fs::create_dir_all(&t1).unwrap();
        fs::create_dir_all(&t2).unwrap();
        fs::write(t1.join("Dockerfile"), "").unwrap();
        fs::write(t2.join("Cargo.toml"), "").unwrap();
        let strategy = crate::strategy::Strategy::new(
            dir.path().to_path_buf(),
            &crate::config::MoldXConfig {
                moldx_dir: PathBuf::from("/nonexistent"),
                strategies_dir: dir.path().to_path_buf(),
                bin_dir_name: "bin".into(),
                template_dir_name: "template".into(),
                templates_dir_name: "templates".into(),
                modules_dir: PathBuf::from("/nonexistent"),
                max_resolution_depth: 20,
            },
        ).unwrap();
        let result = select_template(&strategy, Some("docker"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap().name, "docker");
    }

    #[test]
    fn test_select_template_by_name_not_found() {
        let dir = tempdir().unwrap();
        let tpl_dir = dir.path().join("template");
        fs::create_dir(&tpl_dir).unwrap();
        fs::write(tpl_dir.join("Dockerfile"), "").unwrap();
        let strategy = crate::strategy::Strategy::new(
            dir.path().to_path_buf(),
            &crate::config::MoldXConfig {
                moldx_dir: PathBuf::from("/nonexistent"),
                strategies_dir: dir.path().to_path_buf(),
                bin_dir_name: "bin".into(),
                template_dir_name: "template".into(),
                templates_dir_name: "templates".into(),
                modules_dir: PathBuf::from("/nonexistent"),
                max_resolution_depth: 20,
            },
        ).unwrap();
        let result = select_template(&strategy, Some("nonexistent"));
        assert!(result.is_err());
    }

    #[test]
    fn test_scaffold_template_dir() {
        let dir = tempdir().unwrap();
        let tpl = dir.path().join("template");
        fs::create_dir_all(tpl.join("subdir")).unwrap();
        fs::write(tpl.join("Dockerfile"), "FROM scratch").unwrap();
        fs::write(tpl.join("subdir").join("config.yml"), "key: value").unwrap();
        let target = dir.path().join("module");
        fs::create_dir(&target).unwrap();
        scaffold_template_dir(&tpl, &target).unwrap();
        assert!(target.join("Dockerfile").exists());
        assert!(target.join("subdir").join("config.yml").exists());
        assert_eq!(fs::read_to_string(target.join("Dockerfile")).unwrap(), "FROM scratch");
    }
}
