mod crash;
mod logging;
mod paths;

use m32_domain::BuildInfo;

fn main() {
    crash::install();

    if let Err(error) = logging::init() {
        eprintln!("M32 logging initialization failed: {error}");
        std::process::exit(1);
    }

    let paths = match paths::AppPaths::discover() {
        Ok(paths) => paths,
        Err(error) => {
            tracing::error!(
                target: "m32::storage",
                event = "path_discovery_failed",
                error = %error,
                "M32 runtime path discovery failed"
            );
            std::process::exit(2);
        }
    };

    if let Err(error) = paths.ensure_directories() {
        tracing::error!(
            target: "m32::storage",
            event = "directory_creation_failed",
            error = %error,
            "M32 runtime directory initialization failed"
        );
        std::process::exit(2);
    }

    let info = BuildInfo::current();

    tracing::info!(
        target: "m32::lifecycle",
        event = "app_start",
        app_version = info.app_version,
        product_spec_version = info.product_spec_version,
        spec_bundle_version = info.spec_bundle_version,
        git_commit = info.git_commit,
        wie_commit = info.wie_commit,
        rust_version = info.rust_version,
        build_target = info.target,
        build_profile = info.build_profile,
        "M32 application started"
    );

    tracing::debug!(
        target: "m32::storage",
        event = "runtime_paths_ready",
        "M32 runtime directories are ready"
    );

    crash::trigger_smoke_test_if_requested();
}
