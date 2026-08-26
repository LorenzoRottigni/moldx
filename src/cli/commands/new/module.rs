use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
use walkdir::WalkDir;

use crate::{client::MoldXClient, errors::MoldXError2, template::Template};

/// Scaffolds a new module directory.
///
/// Accepts `<module-path>`, `<profile> <module-path>`, or
/// `<profile> <template> <module-path>`. When a profile is given, the
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
/// Returns [`MoldXError2::NewModuleUsage`] on malformed arguments,
/// [`MoldXError2::ModulePathAlreadyExists`] if the path already exists,
/// [`MoldXError2::ProfileNotFound`] if the profile is unknown, any error
/// raised while selecting a template, and any IO error while scaffolding.
pub fn new_module(client: &MoldXClient, args: Vec<String>) -> Result<()> {
    let (profile_name, template_name, module_index) = match args.len() {
        2 => (None, None, 1),
        3 => (Some(args[1].clone()), None, 2),
        4 => (Some(args[1].clone()), Some(args[2].clone()), 3),
        _ => return Err(MoldXError2::NewModuleUsage.into()),
    };

    let module_path = PathBuf::from(&args[module_index]);
    if module_path.exists() {
        return Err(MoldXError2::ModulePathAlreadyExists { path: module_path }.into());
    }

    fs::create_dir_all(&module_path)?;

    if let Some(profile_name) = profile_name {
        let profile = client
            .profiles
            .iter()
            .find(|p| p.name == profile_name)
            .ok_or_else(|| MoldXError2::ProfileNotFound { name: profile_name })?;
        let template = select_template(profile, template_name.as_deref())?;
        scaffold_template_dir(&template.path, &module_path)?;
        println!(
            "Scaffolded module {} from {} / {} at {}",
            module_path.file_name().and_then(|name| name.to_str()).unwrap_or("module"),
            profile.name,
            template.path.file_name().and_then(|n| n.to_str()).unwrap_or("template"),
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
/// explicit name, the matching template is returned; otherwise the profile
/// must expose exactly one scaffoldable template.
///
/// # Arguments
///
/// * `profile` - The profile to pick a template from.
/// * `template_name` - Optional explicit template name.
///
/// # Returns
///
/// The selected template.
///
/// # Errors
///
/// Returns [`MoldXError2::TemplateNotFound`] if the named template does not
/// exist or contains no files, [`MoldXError2::NoScaffoldableTemplate`] if
/// the profile exposes none, and [`MoldXError2::MultipleTemplates`] if it
/// exposes several without an explicit choice.
fn select_template(profile: &crate::profile::Profile, template_name: Option<&str>) -> Result<Template> {
    // A profile always has exactly one template. Check if it has files.
    if profile.template.file_names.is_empty() {
        return Err(MoldXError2::NoScaffoldableTemplate { name: profile.name.clone() }.into());
    }

    match template_name {
        Some(name) => {
            // When an explicit name is given, it must match the profile name
            if name == profile.name {
                Ok(profile.template.clone())
            } else {
                Err(MoldXError2::TemplateNotFound { profile: profile.name.clone() }.into())
            }
        }
        None => Ok(profile.template.clone()),
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
        let profiles_dir = moldx_dir.join("profiles");
        fs::create_dir_all(&profiles_dir).unwrap();
        let config = crate::config::MoldXConfig {
            moldx_dir,
            profiles_dir,
            profiles_dir_name: "profiles".into(),
            bin_dir_name: "bin".into(),
            template_dir_name: "template".into(),
            templates_dir_name: "templates".into(),
            modules_dir: dir.to_path_buf(),
            max_resolution_depth: 20,
        };
        MoldXClient::new(config).unwrap()
    }

    fn create_profile(dir: &std::path::Path, name: &str) {
        let profile_dir = dir.join(".moldx/profiles").join(name);
        fs::create_dir_all(profile_dir.join("bin")).unwrap();
        fs::create_dir_all(profile_dir.join("template")).unwrap();
    }

    #[test]
    fn test_new_module_no_profile() {
        let dir = tempdir().unwrap();
        let client = make_client(dir.path());
        let module_path = dir.path().join("my-module");
        let result = new_module(&client, vec!["module".into(), module_path.to_str().unwrap().into()]);
        assert!(result.is_ok());
        assert!(module_path.exists());
    }

    #[test]
    fn test_new_module_with_profile() {
        let dir = tempdir().unwrap();
        create_profile(dir.path(), "docker");
        fs::write(dir.path().join(".moldx/profiles/docker/template/Dockerfile"), "FROM scratch").unwrap();
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
    fn test_new_module_profile_not_found() {
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

    fn make_profile_with_template(name: &str, template_files: &[&str]) -> (tempfile::TempDir, crate::profile::Profile) {
        let dir = tempdir().unwrap();
        let profile_dir = dir.path().join(name);
        let bin_dir = profile_dir.join("bin");
        let template_dir = profile_dir.join("template");
        fs::create_dir_all(&bin_dir).unwrap();
        fs::create_dir_all(&template_dir).unwrap();
        for file in template_files {
            fs::write(template_dir.join(file), "").unwrap();
        }
        let config = crate::config::MoldXConfig {
            moldx_dir: PathBuf::from("/nonexistent"),
            profiles_dir: dir.path().to_path_buf(),
            profiles_dir_name: "profiles".into(),
            bin_dir_name: "bin".into(),
            template_dir_name: "template".into(),
            templates_dir_name: "templates".into(),
            modules_dir: PathBuf::from("/nonexistent"),
            max_resolution_depth: 20,
        };
        let profile = crate::profile::Profile::new(&profile_dir, &config).unwrap();
        (dir, profile)
    }

    #[test]
    fn test_select_template_single() {
        let (_dir, profile) = make_profile_with_template("myprofile", &["Dockerfile"]);
        let result = select_template(&profile, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_select_template_empty_template() {
        let (_dir, profile) = make_profile_with_template("myprofile", &[]);
        let result = select_template(&profile, None);
        assert!(result.is_err());
    }

    #[test]
    fn test_select_template_by_name() {
        let (_dir, profile) = make_profile_with_template("docker", &["Dockerfile"]);
        let result = select_template(&profile, Some("docker"));
        assert!(result.is_ok());
    }

    #[test]
    fn test_select_template_by_name_not_found() {
        let (_dir, profile) = make_profile_with_template("docker", &["Dockerfile"]);
        let result = select_template(&profile, Some("nonexistent"));
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
