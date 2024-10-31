use qpmu_api::{export, host::{self, Capture}, Plugin};

struct Echo;

impl Plugin for Echo {
    fn test(name: String) -> Vec<String> {
        let output = host::spawn("echo", &[name], Capture::STDOUT);

        vec![format!("{output:?}")]
    }
}

export!(Echo with_types_in qpmu_api::bindings);
