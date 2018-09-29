extern crate feature_probe;

use feature_probe::Probe;

fn main() {
    let probe = Probe::new();
    if probe.probe_expression("&() as &dyn Sync") {
        println!("cargo:rustc-cfg=has_dyn_trait");
    }
}
