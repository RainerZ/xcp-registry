// Round trip tests: a registry written to a file and loaded again
//
// json_round_trip builds a registry by hand, it needs no feature.
//
// The fixture tests (feature a2l_reader) run on every *.a2l file below tests/fixtures/, no file is named in this code.
// Next to each <name>.a2l there is a <name>.json, the registry as it is expected to be loaded from the A2L file.
// To add a test case, copy an A2L file into the folder or a sub folder. To create or update the .json files, run
//
//     XCP_REGISTRY_UPDATE_FIXTURES=1 cargo test -p xcp_registry --features a2l_reader --test round_trip
//
// and review the result with git diff.

use std::path::{Path, PathBuf};
use xcp_registry::*;

// Empty directory in the system temp directory, unique for this process and test
fn test_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("xcp_registry_test_{}_{}", std::process::id(), name));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_json(reg: &Registry, path: &Path) -> Result<String, String> {
    reg.write_json(&path).map_err(|e| format!("write_json failed: {e}"))?;
    std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

fn load_json(path: &Path) -> Result<Registry, String> {
    let mut reg = Registry::new();
    reg.load_json(&path).map_err(|e| format!("load_json failed: {e}"))?;
    Ok(reg)
}

// Ok if both texts are equal, otherwise the first line which differs
fn same_text(what: &str, expected: &str, actual: &str) -> Result<(), String> {
    let (mut e, mut a) = (expected.lines(), actual.lines());
    for line in 1.. {
        match (e.next(), a.next()) {
            (None, None) => break,
            (e, a) if e == a => {}
            (e, a) => {
                return Err(format!(
                    "{what}, first difference in line {line}:\n  expected: {}\n  actual:   {}",
                    e.unwrap_or("<end>"),
                    a.unwrap_or("<end>")
                ));
            }
        }
    }
    Ok(())
}

// XCP transport and protocol layer parameters, the A2ML mode and the flatten and prefix modes are not part of the JSON file
#[test]
fn json_round_trip() {
    let dir = test_dir("json");
    let mut reg = Registry::new();
    reg.application.set_info("test", "test application", 1);
    reg.application.set_version("V1.2.3", 0x8000_0000);
    reg.event_list.add_event(McEvent::new("task", 0, 1, 10_000_000)).unwrap();
    reg.event_list.add_event(McEvent::new("task", 1, 2, 0)).unwrap();
    reg.cal_seg_list.add_cal_seg("params", Some(1), 16).unwrap();
    reg.add_typedef("Inner", 4).unwrap();
    reg.add_typedef_field(
        "Inner",
        "value",
        McDimType::new(McValueType::Float32Ieee, 1, 1),
        McSupportData::new(McObjectType::Characteristic),
        0,
    )
    .unwrap();
    reg.add_typedef("Params", 16).unwrap();
    let gain = McSupportData::new(McObjectType::Characteristic)
        .set_comment("gain")
        .set_unit("V")
        .set_min(Some(-10.0))
        .set_max(Some(10.0))
        .set_step(Some(0.5));
    reg.add_typedef_field("Params", "gain", McDimType::new(McValueType::Float64Ieee, 1, 1), gain, 0).unwrap();
    reg.add_typedef_field(
        "Params",
        "map",
        McDimType::new(McValueType::Ubyte, 2, 2),
        McSupportData::new(McObjectType::Characteristic),
        8,
    )
    .unwrap();
    let inner = McDimType::new(McValueType::new_typedef("Inner"), 1, 1);
    reg.add_typedef_field("Params", "inner", inner, McSupportData::new(McObjectType::Characteristic), 12)
        .unwrap();
    reg.instance_list
        .add_instance(
            "params",
            McDimType::new(McValueType::new_typedef("Params"), 1, 1),
            McSupportData::new(McObjectType::Characteristic).set_comment("parameters"),
            McAddress::new_calseg_rel("params", 0),
        )
        .unwrap();
    reg.instance_list
        .add_instance(
            "voltage",
            McDimType::new(McValueType::Uword, 1, 1),
            McSupportData::new(McObjectType::Measurement).set_linear(0.5, 1.0, "Volt"),
            McAddress::new_event_abs(1, 0x100),
        )
        .unwrap();
    reg.instance_list
        .add_instance(
            "state",
            McDimType::new(McValueType::Sbyte, 1, 1),
            McSupportData::new(McObjectType::Measurement).set_unit(r#"0 "OFF" 1 "ON""#),
            McAddress::new_event_dyn(0, 2, -4),
        )
        .unwrap();
    reg.instance_list
        .add_instance(
            "samples",
            McDimType::new(McValueType::Slong, 8, 1),
            McSupportData::new(McObjectType::Measurement),
            McAddress::new_event_dyn(0, 2, 8),
        )
        .unwrap();

    let json = write_json(&reg, &dir.join("written.json")).unwrap();
    let loaded = load_json(&dir.join("written.json")).unwrap();
    let json_again = write_json(&loaded, &dir.join("written_again.json")).unwrap();
    same_text("registry changed by JSON round trip", &json, &json_again).unwrap();

    // The content really is in the loaded registry
    assert_eq!(loaded.application.get_name(), "test");
    assert_eq!(loaded.application.get_version(), "V1.2.3");
    assert_eq!(loaded.event_list.find_event("task", 1).unwrap().get_id(), 2);
    assert_eq!(loaded.cal_seg_list.get_cal_seg_index("params"), reg.cal_seg_list.get_cal_seg_index("params"));
    let gain = loaded.typedef_list.find_typedef("Params").unwrap().find_field("gain").unwrap();
    assert_eq!(gain.get_mc_support_data().get_unit(), "V");
    assert_eq!(gain.get_mc_support_data().get_step(), Some(0.5));
    assert_eq!(loaded.typedef_list.find_typedef("Params").unwrap().find_field("inner").unwrap().get_offset(), 12);
    let params = loaded.instance_list.get_instance("params", McObjectType::Characteristic, None).unwrap();
    assert_eq!(params.get_typedef_name(), Some("Params"));
    assert_eq!(*params.get_address(), McAddress::new_calseg_rel("params", 0));
    let voltage = loaded.instance_list.get_instance("voltage", McObjectType::Measurement, Some(1)).unwrap();
    assert_eq!(voltage.get_mc_support_data().get_factor(), Some(0.5));
    assert_eq!(*voltage.get_address(), McAddress::new_event_abs(1, 0x100));
    let state = loaded.instance_list.get_instance("state", McObjectType::Measurement, Some(2)).unwrap();
    assert_eq!(state.unit(), r#"0 "OFF" 1 "ON""#);
    assert_eq!(*state.get_address(), McAddress::new_event_dyn(0, 2, -4));
    assert_eq!(loaded.instance_list.get_instance("samples", McObjectType::Measurement, Some(2)).unwrap().x_dim(), 8);

    std::fs::remove_dir_all(&dir).unwrap();
}

#[cfg(feature = "a2l_reader")]
mod fixtures {
    use super::*;
    use std::panic::{AssertUnwindSafe, catch_unwind};

    const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");
    const UPDATE: &str = "XCP_REGISTRY_UPDATE_FIXTURES";

    fn a2l_files(dir: &Path, files: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display())) {
            let path = entry.unwrap().path();
            if path.is_dir() {
                a2l_files(&path, files);
            } else if path.extension().is_some_and(|e| e == "a2l") {
                files.push(path);
            }
        }
    }

    // Run check on every A2L file below tests/fixtures/ and report all files which failed
    fn for_each_fixture(test: &str, check: impl Fn(&Path, &Path) -> Result<(), String>) {
        let mut files = Vec::new();
        a2l_files(Path::new(FIXTURES), &mut files);
        files.sort();
        assert!(!files.is_empty(), "no A2L files found in {FIXTURES}");

        let dir = test_dir(test);
        let mut failures = Vec::new();
        for (i, file) in files.iter().enumerate() {
            let file_dir = dir.join(i.to_string());
            std::fs::create_dir_all(&file_dir).unwrap();
            let result = catch_unwind(AssertUnwindSafe(|| check(file, &file_dir))).unwrap_or_else(|_| Err("panicked, see the panic message above".to_string()));
            if let Err(e) = result {
                let name = file.strip_prefix(FIXTURES).unwrap().display();
                failures.push(format!("{name}: {e}\n  files written by the test: {}", file_dir.display()));
            }
        }
        assert!(failures.is_empty(), "{} of {} fixtures failed\n\n{}\n", failures.len(), files.len(), failures.join("\n\n"));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    // The fixtures include XCP_104.aml, a2lfile finds it in the working directory of the test, which is the crate folder
    fn load_a2l(path: &Path) -> Result<Registry, String> {
        let mut reg = Registry::new();
        reg.load_a2l(&path, false, true, false, false).map_err(|e| format!("load_a2l failed: {e}"))?;
        Ok(reg)
    }

    fn write_a2l(reg: &mut Registry, path: &Path) -> Result<String, String> {
        // Self-contained file, no XCP_104.aml needed next to it
        reg.set_a2ml_mode(McA2mlMode::Embed);
        reg.write_a2l(&path, "", reg.application.get_name(), "", "module", "fixture", false)
            .map_err(|e| format!("write_a2l failed: {e}"))?;
        std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
    }

    // Compare the loaded registry with the stored <name>.json, or store it
    fn check_expected(file: &Path, reg: &Registry, json: &str) -> Result<(), String> {
        let expected = file.with_extension("json");
        if std::env::var_os(UPDATE).is_some() {
            return reg.write_json(&expected).map_err(|e| format!("{}: {e}", expected.display()));
        }
        let name = expected.file_name().unwrap().to_string_lossy();
        let text = std::fs::read_to_string(&expected).map_err(|e| format!("{name}: {e}\n  run the test with {UPDATE}=1 to create it"))?;
        same_text(&format!("loaded registry differs from {name}"), &text, json).map_err(|e| format!("{e}\n  if the change is intended, run the test with {UPDATE}=1"))
    }

    // Every fixture loads as stored in its .json file, survives a JSON round trip unchanged,
    // and can be written as a valid A2L file which does not change when it is loaded and written again
    #[test]
    fn fixtures() {
        for_each_fixture("fixtures", |file, dir| {
            let mut reg = load_a2l(file)?;
            let json = write_json(&reg, &dir.join("loaded.json"))?;
            check_expected(file, &reg, &json)?;

            let from_json = load_json(&dir.join("loaded.json"))?;
            let json_again = write_json(&from_json, &dir.join("loaded_again.json"))?;
            same_text("registry changed by JSON round trip", &json, &json_again)?;

            let a2l = write_a2l(&mut reg, &dir.join("written.a2l"))?;
            reg.check_a2l(&dir.join("written.a2l")).map_err(|e| format!("written A2L file is not valid: {e}"))?;
            let mut from_a2l = load_a2l(&dir.join("written.a2l"))?;
            let a2l_again = write_a2l(&mut from_a2l, &dir.join("written_again.a2l"))?;
            same_text("A2L file changed when loaded and written again", &a2l, &a2l_again)
        });
    }

    // The registry loaded from the written A2L file is the registry loaded from the fixture, the order of the instances may differ
    #[test]
    #[ignore = "a CHARACTERISTIC with an event is loaded but not written"]
    fn fixtures_a2l_round_trip_keeps_registry() {
        for_each_fixture("fixtures_keeps_registry", |file, dir| {
            let mut reg = load_a2l(file)?;
            reg.instance_list.sort_by_name_and_event();
            let json = write_json(&reg, &dir.join("loaded.json"))?;
            write_a2l(&mut reg, &dir.join("written.a2l"))?;
            let mut from_a2l = load_a2l(&dir.join("written.a2l"))?;
            from_a2l.instance_list.sort_by_name_and_event();
            let json_from_a2l = write_json(&from_a2l, &dir.join("from_a2l.json"))?;
            same_text("registry changed by A2L round trip", &json, &json_from_a2l)
        });
    }
}
