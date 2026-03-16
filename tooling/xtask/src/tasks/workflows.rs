use anyhow::{Context, Result};
use clap::Parser;
use gh_workflow::Workflow;
use std::fs;
use std::path::{Path, PathBuf};

mod after_release;
mod autofix_pr;
mod bump_patch_version;
mod cherry_pick;
mod compare_perf;
mod danger;
mod extension_bump;
mod extension_release;
mod extension_tests;
mod extension_workflow_rollout;
mod extensions;
mod nix_build;
mod release_nightly;
mod run_bundling;

mod release;
mod run_agent_evals;
mod run_tests;
mod runners;
mod steps;
mod vars;

#[derive(Parser)]
pub struct GenerateWorkflowArgs {}

struct WorkflowFile {
    source: fn() -> Workflow,
    r#type: WorkflowType,
}

impl WorkflowFile {
    fn zed_custom(f: fn() -> Workflow) -> WorkflowFile {
        WorkflowFile {
            source: f,
            r#type: WorkflowType::Zed,
        }
    }

    fn extension(f: fn() -> Workflow) -> WorkflowFile {
        WorkflowFile {
            source: f,
            r#type: WorkflowType::ExtensionCI,
        }
    }

    fn extension_shared(f: fn() -> Workflow) -> WorkflowFile {
        WorkflowFile {
            source: f,
            r#type: WorkflowType::ExtensionsShared,
        }
    }

    fn generate_file(&self) -> Result<()> {
        let workflow = (self.source)();
        let workflow_folder = self.r#type.folder_path();

        fs::create_dir_all(&workflow_folder).with_context(|| {
            format!("Failed to create directory: {}", workflow_folder.display())
        })?;

        let workflow_name = workflow
            .name
            .as_ref()
            .expect("Workflow must have a name at this point");
        let filename = format!(
            "{}.yml",
            workflow_name.rsplit("::").next().unwrap_or(workflow_name)
        );

        let workflow_path = workflow_folder.join(filename);

        let content = workflow
            .to_string()
            .map_err(|e| anyhow::anyhow!("{:?}: {:?}", workflow_path, e))?;

        let disclaimer = self.r#type.disclaimer(workflow_name);

        let content = [disclaimer, content].join("\n");
        fs::write(&workflow_path, content).map_err(Into::into)
    }
}

#[derive(PartialEq, Eq)]
enum WorkflowType {
    /// Workflows living in the Zed repository
    Zed,
    /// Workflows living in the `zed_custom-extensions/workflows` repository that are
    /// required workflows for PRs to the extension organization
    ExtensionCI,
    /// Workflows living in each of the extensions to perform checks and version
    /// bumps until a better, more centralized system for that is in place.
    ExtensionsShared,
}

impl WorkflowType {
    fn disclaimer(&self, workflow_name: &str) -> String {
        format!(
            concat!(
                "# Generated from xtask::workflows::{}{}\n",
                "# Rebuild with `cargo xtask workflows`.",
            ),
            workflow_name,
            (*self != WorkflowType::Zed)
                .then_some(" within the zed_custom repository.")
                .unwrap_or_default(),
        )
    }

    fn folder_path(&self) -> PathBuf {
        match self {
            WorkflowType::Zed => PathBuf::from(".github/workflows"),
            WorkflowType::ExtensionCI => PathBuf::from("extensions/workflows"),
            WorkflowType::ExtensionsShared => PathBuf::from("extensions/workflows/shared"),
        }
    }
}

pub fn run_workflows(_: GenerateWorkflowArgs) -> Result<()> {
    if !Path::new("crates/zed-custom/").is_dir() {
        anyhow::bail!("xtask workflows must be ran from the project root");
    }

    let workflows = [
        WorkflowFile::zed_custom(after_release::after_release),
        WorkflowFile::zed_custom(autofix_pr::autofix_pr),
        WorkflowFile::zed_custom(bump_patch_version::bump_patch_version),
        WorkflowFile::zed_custom(cherry_pick::cherry_pick),
        WorkflowFile::zed_custom(compare_perf::compare_perf),
        WorkflowFile::zed_custom(danger::danger),
        WorkflowFile::zed_custom(extension_bump::extension_bump),
        WorkflowFile::zed_custom(extension_release::extension_release),
        WorkflowFile::zed_custom(extension_tests::extension_tests),
        WorkflowFile::zed_custom(extension_workflow_rollout::extension_workflow_rollout),
        WorkflowFile::zed_custom(release::release_mac),
        WorkflowFile::zed_custom(release::release_linux),
        WorkflowFile::zed_custom(release_nightly::release_nightly),
        WorkflowFile::zed_custom(run_agent_evals::run_agent_evals),
        WorkflowFile::zed_custom(run_agent_evals::run_cron_unit_evals),
        WorkflowFile::zed_custom(run_agent_evals::run_unit_evals),
        WorkflowFile::zed_custom(run_bundling::run_bundling),
        WorkflowFile::zed_custom(run_tests::run_tests),
        /* workflows used for CI/CD in extension repositories */
        WorkflowFile::extension(extensions::run_tests::run_tests),
        WorkflowFile::extension_shared(extensions::bump_version::bump_version),
        WorkflowFile::extension_shared(extensions::release_version::release_version),
    ];

    for workflow_file in workflows {
        workflow_file.generate_file()?;
    }

    Ok(())
}
