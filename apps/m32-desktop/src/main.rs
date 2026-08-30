mod logging;

use m32_domain::BuildInfo;

fn main() {
    if let Err(error) = logging::init() {
        eprintln!("M32 logging initialization failed: {error}");
        std::process::exit(1);
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
}
