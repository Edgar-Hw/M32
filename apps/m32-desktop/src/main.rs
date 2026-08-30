use m32_domain::BuildInfo;

fn main() {
    let info = BuildInfo::current();

    println!("M32 BuildInfo");
    println!("app_version={}", info.app_version);
    println!("product_spec_version={}", info.product_spec_version);
    println!("spec_bundle_version={}", info.spec_bundle_version);
    println!("git_commit={}", info.git_commit);
    println!("wie_commit={}", info.wie_commit);
    println!("rust_version={}", info.rust_version);
    println!("target={}", info.target);
    println!("build_profile={}", info.build_profile);
}
